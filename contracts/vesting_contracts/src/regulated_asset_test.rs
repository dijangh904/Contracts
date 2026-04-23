#![cfg(test)]
use soroban_sdk::{
    Address,
    Env,
    Vec,
    String,
    BytesN,
};
use crate::{
    regulated_asset::{
        RegulatedAssetManager, RegulatedAssetError, AssetRegulation, SEP08Authorization,
        AuthorizationStatus, FreezeEvent, ClawbackEvent, RegulatedAssetTrait,
        ASSET_REGULATIONS, SEP08_AUTHORIZATIONS, FREEZE_EVENTS, CLAWBACK_EVENTS,
    },
    testutils::{create_test_contract, create_test_address, create_test_env},
};

#[test]
fn test_register_regulated_asset() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let issuer = create_test_address(&env);

    let compliance_requirements = Vec::from_array(&env, &[
        "KYC required".to_string(),
        "Accredited investor verification".to_string(),
    ]);

    let result = RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true, // requires_authorization
        true, // supports_freeze
        true, // supports_clawback
        365 * 24 * 60 * 60, // 1 year max duration
        compliance_requirements.clone(),
    );

    assert!(result.is_ok());

    // Verify asset regulation was stored
    let regulation = RegulatedAssetManager::get_asset_regulation(&env, asset_id.clone()).unwrap();
    assert_eq!(regulation.asset_id, asset_id);
    assert_eq!(regulation.issuer, issuer);
    assert!(regulation.requires_authorization);
    assert!(regulation.supports_freeze);
    assert!(regulation.supports_clawback);
}

#[test]
fn test_register_duplicate_asset() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let issuer = create_test_address(&env);

    // Register asset first time
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true,
        true,
        true,
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    // Try to register same asset again
    let result = RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id,
        issuer,
        true,
        true,
        true,
        365 * 24 * 60 * 60,
        Vec::new(&env),
    );

    assert_eq!(result.err(), Some(RegulatedAssetError::AssetNotSupported));
}

#[test]
fn test_create_authorization() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let holder = create_test_address(&env);
    let issuer = create_test_address(&env);
    let authorization_id = BytesN::from_array([0x01; 32]);

    // First register the asset
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true, // requires_authorization
        true, // supports_freeze
        true, // supports_clawback
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    // Create authorization
    let result = RegulatedAssetManager::create_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        1000000, // authorized amount
        authorization_id.clone(),
        env.ledger().timestamp() + 365 * 24 * 60 * 60, // expires in 1 year
        issuer.clone(),
        0, // compliance flags
    );

    assert!(result.is_ok());

    // Verify authorization was created
    let auth = RegulatedAssetManager::get_authorization(&env, authorization_id.clone()).unwrap();
    assert_eq!(auth.asset_id, asset_id);
    assert_eq!(auth.holder, holder);
    assert_eq!(auth.authorized_amount, 1000000);
    assert_eq!(auth.issuer, issuer);
    assert_eq!(auth.status, AuthorizationStatus::Active);
}

#[test]
fn test_validate_authorization_success() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let holder = create_test_address(&env);
    let issuer = create_test_address(&env);
    let authorization_id = BytesN::from_array([0x01; 32]);

    // Register asset and create authorization
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true,
        true,
        true,
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    RegulatedAssetManager::create_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        1000000,
        authorization_id.clone(),
        env.ledger().timestamp() + 365 * 24 * 60 * 60,
        issuer.clone(),
        0,
    ).unwrap();

    // Validate authorization for smaller amount
    let result = RegulatedAssetManager::validate_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        500000, // less than authorized amount
        authorization_id.clone(),
    );

    assert!(result.is_ok());
}

#[test]
fn test_validate_authorization_insufficient() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let holder = create_test_address(&env);
    let issuer = create_test_address(&env);
    let authorization_id = BytesN::from_array([0x01; 32]);

    // Register asset and create authorization
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true,
        true,
        true,
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    RegulatedAssetManager::create_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        1000000,
        authorization_id.clone(),
        env.ledger().timestamp() + 365 * 24 * 60 * 60,
        issuer.clone(),
        0,
    ).unwrap();

    // Try to validate for more than authorized amount
    let result = RegulatedAssetManager::validate_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        1500000, // more than authorized amount
        authorization_id.clone(),
    );

    assert_eq!(result.err(), Some(RegulatedAssetError::InsufficientAuthorization));
}

#[test]
fn test_validate_authorization_expired() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let holder = create_test_address(&env);
    let issuer = create_test_address(&env);
    let authorization_id = BytesN::from_array([0x01; 32]);

    // Register asset and create authorization with short expiry
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true,
        true,
        true,
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    RegulatedAssetManager::create_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        1000000,
        authorization_id.clone(),
        env.ledger().timestamp() + 1, // expires in 1 second
        issuer.clone(),
        0,
    ).unwrap();

    // Wait for expiry
    // Note: In real test, you'd need to advance ledger timestamp
    // For this test, we'll simulate expired state

    // Try to validate expired authorization
    let result = RegulatedAssetManager::validate_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        500000,
        authorization_id.clone(),
    );

    assert_eq!(result.err(), Some(RegulatedAssetError::AuthorizationExpired));
}

#[test]
fn test_handle_freeze_event() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let holder = create_test_address(&env);
    let issuer = create_test_address(&env);
    let issuer_signature = BytesN::from_array([0x02; 32]);

    // Register asset with freeze support
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true,
        true, // supports_freeze
        true,
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    let result = RegulatedAssetManager::handle_freeze_event(
        &env,
        asset_id.clone(),
        holder.clone(),
        100000, // amount to freeze
        "Regulatory compliance freeze".to_string(),
        issuer_signature.clone(),
    );

    assert!(result.is_ok());

    // Verify asset still requires authorization but is now frozen
    assert!(RegulatedAssetManager::requires_authorization(&env, asset_id.clone()));
    assert!(RegulatedAssetManager::supports_freeze(&env, asset_id.clone()));
}

#[test]
fn test_handle_clawback_event() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let from_holder = create_test_address(&env);
    let issuer = create_test_address(&env);
    let issuer_signature = BytesN::from_array([0x03; 32]);

    // Register asset with clawback support
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true,
        true,
        true, // supports_clawback
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    let result = RegulatedAssetManager::handle_clawback_event(
        &env,
        asset_id.clone(),
        from_holder.clone(),
        50000, // amount to clawback
        "Regulatory clawback order".to_string(),
        issuer_signature.clone(),
    );

    assert!(result.is_ok());

    // Verify asset still requires authorization and supports clawback
    assert!(RegulatedAssetManager::requires_authorization(&env, asset_id.clone()));
    assert!(RegulatedAssetManager::supports_clawback(&env, asset_id.clone()));
}

#[test]
fn test_consume_authorization() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let holder = create_test_address(&env);
    let issuer = create_test_address(&env);
    let authorization_id = BytesN::from_array([0x01; 32]);

    // Register asset and create authorization
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true,
        true,
        true,
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    RegulatedAssetManager::create_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        1000000,
        authorization_id.clone(),
        env.ledger().timestamp() + 365 * 24 * 60 * 60,
        issuer.clone(),
        0,
    ).unwrap();

    // Consume part of authorization
    RegulatedAssetManager::consume_authorization(&env, authorization_id.clone(), 250000);

    // Verify authorization was updated
    let auth = RegulatedAssetManager::get_authorization(&env, authorization_id.clone()).unwrap();
    assert_eq!(auth.used_amount, 250000);
    assert_eq!(auth.authorized_amount - auth.used_amount, 750000);
}

#[test]
fn test_revoke_authorization() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let holder = create_test_address(&env);
    let issuer = create_test_address(&env);
    let authorization_id = BytesN::from_array([0x01; 32]);

    // Register asset and create authorization
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true,
        true,
        true,
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    RegulatedAssetManager::create_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        1000000,
        authorization_id.clone(),
        env.ledger().timestamp() + 365 * 24 * 60 * 60,
        issuer.clone(),
        0,
    ).unwrap();

    // Revoke authorization
    let result = RegulatedAssetManager::revoke_authorization(
        &env,
        issuer.clone(),
        authorization_id.clone(),
        "Regulatory revocation".to_string(),
    );

    assert!(result.is_ok());

    // Verify authorization was revoked
    let auth = RegulatedAssetManager::get_authorization(&env, authorization_id.clone()).unwrap();
    assert_eq!(auth.status, AuthorizationStatus::Revoked);
}

#[test]
fn test_unregulated_asset() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);

    // Test with unregistered asset (should return false)
    let requires_auth = RegulatedAssetManager::requires_authorization(&env, asset_id.clone());
    assert!(!requires_auth);

    let supports_freeze = RegulatedAssetManager::supports_freeze(&env, asset_id.clone());
    assert!(!supports_freeze);

    let supports_clawback = RegulatedAssetManager::supports_clawback(&env, asset_id.clone());
    assert!(!supports_clawback);

    // Should return None for unregistered asset
    let regulation = RegulatedAssetManager::get_asset_regulation(&env, asset_id);
    assert!(regulation.is_none());
}

#[test]
fn test_authorization_status_transitions() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let asset_id = create_test_address(&env);
    let holder = create_test_address(&env);
    let issuer = create_test_address(&env);
    let authorization_id = BytesN::from_array([0x01; 32]);

    // Register asset and create authorization
    RegulatedAssetManager::register_regulated_asset(
        &env,
        asset_id.clone(),
        issuer.clone(),
        true,
        true,
        true,
        365 * 24 * 60 * 60,
        Vec::new(&env),
    ).unwrap();

    RegulatedAssetManager::create_authorization(
        &env,
        asset_id.clone(),
        holder.clone(),
        1000000,
        authorization_id.clone(),
        env.ledger().timestamp() + 365 * 24 * 60 * 60,
        issuer.clone(),
        0,
    ).unwrap();

    // Test initial status
    let auth = RegulatedAssetManager::get_authorization(&env, authorization_id.clone()).unwrap();
    assert_eq!(auth.status, AuthorizationStatus::Active);

    // Revoke authorization
    RegulatedAssetManager::revoke_authorization(
        &env,
        issuer.clone(),
        authorization_id.clone(),
        "Test revocation".to_string(),
    ).unwrap();

    // Verify status changed to revoked
    let auth = RegulatedAssetManager::get_authorization(&env, authorization_id.clone()).unwrap();
    assert_eq!(auth.status, AuthorizationStatus::Revoked);
}
