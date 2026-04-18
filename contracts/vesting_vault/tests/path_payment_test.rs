#![cfg(test)]

use soroban_sdk::{symbol_short, Address, Env, Vec, String, Symbol, IntoVal, Val, Error};
use soroban_sdk::testutils::Address as _;
use vesting_vault::{VestingVault, VestingVaultClient, PathPaymentConfig, PathPaymentSimulation, PathPaymentClaimEvent};

#[test]
fn test_configure_path_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let intermediate_asset = Address::generate(&env);
    
    let mut path = Vec::new(&env);
    path.push_back(intermediate_asset);
    
    let min_destination_amount = 1000i128;
    
    // Configure path payment
    client.configure_path_payment(
        &admin,
        &usdc_asset,
        &min_destination_amount,
        &path
    );
    
    // Verify configuration
    let config = client.get_path_payment_config();
    assert!(config.is_some());
    
    let config = config.unwrap();
    assert_eq!(config.destination_asset, usdc_asset);
    assert_eq!(config.min_destination_amount, min_destination_amount);
    assert_eq!(config.path, path);
    assert!(config.enabled);
}

#[test]
fn test_disable_path_payment() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Configure path payment first
    client.configure_path_payment(&admin, &usdc_asset, &1000i128, &path);
    
    // Disable it
    client.disable_path_payment(&admin);
    
    // Verify it's disabled
    let config = client.get_path_payment_config();
    assert!(config.is_some());
    assert!(!config.unwrap().enabled);
}

#[test]
fn test_claim_with_path_payment_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Configure path payment
    client.configure_path_payment(&admin, &usdc_asset, &950i128, &path);
    
    // Execute claim with path payment
    client.claim_with_path_payment(
        &user,
        &1u32,
        &1000i128,
        &Some(950i128)
    );
}

#[test]
fn test_claim_with_path_payment_not_configured() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    
    // Try to claim without configuration
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "claim_with_path_payment"),
        (user.clone(), 1u32, 1000i128, Some(950i128)).into_val(&env),
    );
    
    assert!(result.is_err());
}

#[test]
fn test_claim_with_path_payment_disabled() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Configure and then disable path payment
    client.configure_path_payment(&admin, &usdc_asset, &950i128, &path);
    client.disable_path_payment(&admin);
    
    // Try to claim while disabled
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "claim_with_path_payment"),
        (user.clone(), 1u32, 1000i128, Some(950i128)).into_val(&env),
    );
    
    assert!(result.is_err());
}

#[test]
fn test_claim_with_path_payment_insufficient_minimum() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Configure path payment
    client.configure_path_payment(&admin, &usdc_asset, &950i128, &path);
    
    // Try to claim with insufficient minimum
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "claim_with_path_payment"),
        (user.clone(), 1u32, 1000i128, Some(999i128)).into_val(&env),
    );
    
    assert!(result.is_err());
}

#[test]
fn test_path_payment_zero_minimum_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Configure path payment
    client.configure_path_payment(&admin, &usdc_asset, &950i128, &path);
    
    // Try to claim with zero minimum amount
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "claim_with_path_payment"),
        (user.clone(), 1u32, 1000i128, Some(0i128)).into_val(&env),
    );
    
    assert!(result.is_err());
}
