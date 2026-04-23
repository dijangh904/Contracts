#![cfg(test)]

use soroban_sdk::{Address, Env};
use vesting_vault::VestingVault;

#[test]
fn test_configure_lockup() {
    let env = Env::default();
    let admin = Address::random(&env);
    let lockup_token_address = Address::random(&env);
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64; // 1 day
    
    VestingVault::configure_lockup(
        env.clone(), 
        admin.clone(), 
        vesting_id, 
        lockup_duration, 
        lockup_token_address.clone()
    );
    
    let config = VestingVault::get_lockup_config(env.clone(), vesting_id).unwrap();
    assert_eq!(config.vesting_id, vesting_id);
    assert_eq!(config.lockup_duration_seconds, lockup_duration);
    assert!(config.enabled);
    assert_eq!(config.lockup_token_address, lockup_token_address);
}

#[test]
fn test_disable_lockup() {
    let env = Env::default();
    let admin = Address::random(&env);
    let lockup_token_address = Address::random(&env);
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64;
    
    // Configure lockup
    VestingVault::configure_lockup(
        env.clone(), 
        admin.clone(), 
        vesting_id, 
        lockup_duration, 
        lockup_token_address
    );
    
    // Verify it's configured
    let config = VestingVault::get_lockup_config(env.clone(), vesting_id);
    assert!(config.is_some());
    
    // Disable lockup
    VestingVault::disable_lockup(env.clone(), admin.clone(), vesting_id);
    
    // Verify it's removed
    let config = VestingVault::get_lockup_config(env.clone(), vesting_id);
    assert!(config.is_none());
}

#[test]
fn test_claim_with_lockup_issued() {
    let env = Env::default();
    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lockup_token_address = Address::random(&env);
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64;
    let amount = 1000i128;
    
    // Configure lockup
    VestingVault::configure_lockup(
        env.clone(), 
        admin.clone(), 
        vesting_id, 
        lockup_duration, 
        lockup_token_address
    );
    
    // Claim with lockup - this should issue wrapped tokens
    VestingVault::claim_with_lockup(env.clone(), user.clone(), vesting_id, amount);
    
    // Verify claim was recorded in history
    let claims = VestingVault::get_all_claims(env.clone());
    assert_eq!(claims.len(), 1);
    
    let claim = claims.get(0).unwrap();
    assert_eq!(claim.beneficiary, user);
    assert_eq!(claim.amount, amount);
    assert_eq!(claim.vesting_id, vesting_id);
}

#[test]
fn test_claim_without_lockup_normal_flow() {
    let env = Env::default();
    let user = Address::random(&env);
    
    let vesting_id = 1u32;
    let amount = 1000i128;
    
    // Claim without lockup configuration - should work normally
    VestingVault::claim_with_lockup(env.clone(), user.clone(), vesting_id, amount);
    
    // Verify claim was recorded in history
    let claims = VestingVault::get_all_claims(env.clone());
    assert_eq!(claims.len(), 1);
    
    let claim = claims.get(0).unwrap();
    assert_eq!(claim.beneficiary, user);
    assert_eq!(claim.amount, amount);
    assert_eq!(claim.vesting_id, vesting_id);
}

#[test]
fn test_is_user_unlocked_no_lockup() {
    let env = Env::default();
    let user = Address::random(&env);
    let vesting_id = 1u32;
    
    // Without lockup configuration, should return true
    let is_unlocked = VestingVault::is_user_unlocked(env.clone(), user.clone(), vesting_id);
    assert!(is_unlocked);
}

#[test]
fn test_is_user_unlocked_with_lockup() {
    let env = Env::default();
    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lockup_token_address = Address::random(&env);
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64;
    
    // Configure lockup
    VestingVault::configure_lockup(
        env.clone(), 
        admin.clone(), 
        vesting_id, 
        lockup_duration, 
        lockup_token_address
    );
    
    // With lockup configuration, should return false (placeholder behavior)
    let is_unlocked = VestingVault::is_user_unlocked(env.clone(), user.clone(), vesting_id);
    assert!(!is_unlocked);
}

#[test]
fn test_get_user_unlock_time_no_lockup() {
    let env = Env::default();
    let user = Address::random(&env);
    let vesting_id = 1u32;
    
    // Without lockup configuration, should return None
    let unlock_time = VestingVault::get_user_unlock_time(env.clone(), user.clone(), vesting_id);
    assert!(unlock_time.is_none());
}

#[test]
fn test_get_user_unlock_time_with_lockup() {
    let env = Env::default();
    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lockup_token_address = Address::random(&env);
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64;
    
    // Configure lockup
    VestingVault::configure_lockup(
        env.clone(), 
        admin.clone(), 
        vesting_id, 
        lockup_duration, 
        lockup_token_address
    );
    
    // With lockup configuration, should return Some(unlock_time)
    let unlock_time = VestingVault::get_user_unlock_time(env.clone(), user.clone(), vesting_id);
    assert!(unlock_time.is_some());
    
    let expected_time = env.ledger().timestamp() + lockup_duration;
    assert_eq!(unlock_time.unwrap(), expected_time);
}

#[test]
fn test_multiple_vesting_ids_lockup_configuration() {
    let env = Env::default();
    let admin = Address::random(&env);
    let lockup_token_address = Address::random(&env);
    
    let vesting_id_1 = 1u32;
    let vesting_id_2 = 2u32;
    let lockup_duration_1 = 86400u64; // 1 day
    let lockup_duration_2 = 172800u64; // 2 days
    
    // Configure different lockup periods
    VestingVault::configure_lockup(
        env.clone(), 
        admin.clone(), 
        vesting_id_1, 
        lockup_duration_1, 
        lockup_token_address.clone()
    );
    
    VestingVault::configure_lockup(
        env.clone(), 
        admin.clone(), 
        vesting_id_2, 
        lockup_duration_2, 
        lockup_token_address
    );
    
    // Verify both configurations exist
    let config_1 = VestingVault::get_lockup_config(env.clone(), vesting_id_1).unwrap();
    assert_eq!(config_1.lockup_duration_seconds, lockup_duration_1);
    
    let config_2 = VestingVault::get_lockup_config(env.clone(), vesting_id_2).unwrap();
    assert_eq!(config_2.lockup_duration_seconds, lockup_duration_2);
}

#[test]
fn test_claim_with_lockup_respects_existing_features() {
    let env = Env::default();
    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lockup_token_address = Address::random(&env);
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64;
    let amount = 1000i128;
    
    // Initialize auditors for emergency pause
    let auditors = vec![
        Address::random(&env),
        Address::random(&env), 
        Address::random(&env)
    ];
    VestingVault::initialize_auditors(env.clone(), admin.clone(), auditors);
    
    // Configure lockup
    VestingVault::configure_lockup(
        env.clone(), 
        admin.clone(), 
        vesting_id, 
        lockup_duration, 
        lockup_token_address
    );
    
    // Claim should work normally when not paused
    VestingVault::claim_with_lockup(env.clone(), user.clone(), vesting_id, amount);
    
    // Verify claim was recorded
    let claims = VestingVault::get_all_claims(env.clone());
    assert_eq!(claims.len(), 1);
}

#[test]
fn test_claim_with_lockup_disabled_vesting() {
    let env = Env::default();
    let admin = Address::random(&env);
    let user = Address::random(&env);
    let lockup_token_address = Address::random(&env);
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64;
    let amount = 1000i128;
    
    // Configure lockup
    VestingVault::configure_lockup(
        env.clone(), 
        admin.clone(), 
        vesting_id, 
        lockup_duration, 
        lockup_token_address
    );
    
    // Disable lockup
    VestingVault::disable_lockup(env.clone(), admin.clone(), vesting_id);
    
    // Claim should work normally without lockup
    VestingVault::claim_with_lockup(env.clone(), user.clone(), vesting_id, amount);
    
    // Verify claim was recorded
    let claims = VestingVault::get_all_claims(env.clone());
    assert_eq!(claims.len(), 1);
}
