#![cfg(test)]

use soroban_sdk::{Address, Env, BytesN};
use crate::{LockupToken, storage, types::*};

#[test]
fn test_initialize() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token.clone());
    
    // Verify admin is set
    assert_eq!(storage::get_admin(&env), Some(admin));
    
    // Verify underlying token is set
    assert_eq!(storage::get_underlying_token(&env), Some(underlying_token));
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_initialize_twice() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token.clone());
    LockupToken::initialize(env.clone(), admin, underlying_token);
}

#[test]
fn test_configure_lockup() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64; // 1 day
    
    LockupToken::configure_lockup(env.clone(), admin.clone(), vesting_id, lockup_duration);
    
    let config = LockupToken::get_lockup_config(env.clone(), vesting_id).unwrap();
    assert_eq!(config.vesting_id, vesting_id);
    assert_eq!(config.lockup_duration_seconds, lockup_duration);
    assert!(config.enabled);
}

#[test]
fn test_issue_wrapped_tokens() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    let minter = Address::random(&env);
    let user = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    LockupToken::add_authorized_minter(env.clone(), admin.clone(), minter.clone());
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64;
    let amount = 1000i128;
    
    LockupToken::configure_lockup(env.clone(), admin.clone(), vesting_id, lockup_duration);
    
    // Issue wrapped tokens
    LockupToken::issue_wrapped_tokens(env.clone(), minter.clone(), user.clone(), vesting_id, amount);
    
    // Check wrapped balance
    let balance = LockupToken::wrapped_balance(env.clone(), user.clone());
    assert_eq!(balance, amount);
    
    // Check lockup info
    let lockup_info = LockupToken::get_lockup_info(env.clone(), user.clone(), vesting_id).unwrap();
    assert_eq!(lockup_info.vesting_id, vesting_id);
    assert_eq!(lockup_info.amount, amount);
    assert!(!lockup_info.is_unwrapped);
    
    let current_time = env.ledger().timestamp();
    assert_eq!(lockup_info.locked_at, current_time);
    assert_eq!(lockup_info.unlock_time, current_time + lockup_duration);
}

#[test]
#[should_panic(expected = "Not an authorized minter")]
fn test_issue_wrapped_tokens_unauthorized() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    let unauthorized_minter = Address::random(&env);
    let user = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    
    let vesting_id = 1u32;
    let amount = 1000i128;
    
    // Try to issue tokens without authorization
    LockupToken::issue_wrapped_tokens(env.clone(), unauthorized_minter.clone(), user.clone(), vesting_id, amount);
}

#[test]
fn test_unwrap_tokens_after_lockup() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    let minter = Address::random(&env);
    let user = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    LockupToken::add_authorized_minter(env.clone(), admin.clone(), minter.clone());
    
    let vesting_id = 1u32;
    let lockup_duration = 1u64; // 1 second for testing
    let amount = 1000i128;
    
    LockupToken::configure_lockup(env.clone(), admin.clone(), vesting_id, lockup_duration);
    LockupToken::issue_wrapped_tokens(env.clone(), minter.clone(), user.clone(), vesting_id, amount);
    
    // Advance time past lockup period
    env.ledger().set_timestamp(env.ledger().timestamp() + lockup_duration + 1);
    
    // Unwrap tokens
    LockupToken::unwrap_tokens(env.clone(), user.clone(), vesting_id, amount);
    
    // Check wrapped balance is zero
    let balance = LockupToken::wrapped_balance(env.clone(), user.clone());
    assert_eq!(balance, 0);
    
    // Check lockup info is updated
    let lockup_info = LockupToken::get_lockup_info(env.clone(), user.clone(), vesting_id).unwrap();
    assert_eq!(lockup_info.amount, 0);
    assert!(lockup_info.is_unwrapped);
}

#[test]
#[should_panic(expected = "Tokens are still locked")]
fn test_unwrap_tokens_during_lockup() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    let minter = Address::random(&env);
    let user = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    LockupToken::add_authorized_minter(env.clone(), admin.clone(), minter.clone());
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64; // 1 day
    let amount = 1000i128;
    
    LockupToken::configure_lockup(env.clone(), admin.clone(), vesting_id, lockup_duration);
    LockupToken::issue_wrapped_tokens(env.clone(), minter.clone(), user.clone(), vesting_id, amount);
    
    // Try to unwrap before lockup period expires
    LockupToken::unwrap_tokens(env.clone(), user.clone(), vesting_id, amount);
}

#[test]
#[should_panic(expected = "Insufficient wrapped token balance")]
fn test_unwrap_tokens_insufficient_balance() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    let minter = Address::random(&env);
    let user = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    LockupToken::add_authorized_minter(env.clone(), admin.clone(), minter.clone());
    
    let vesting_id = 1u32;
    let lockup_duration = 1u64; // 1 second for testing
    let amount = 1000i128;
    let unwrap_amount = 2000i128; // More than issued
    
    LockupToken::configure_lockup(env.clone(), admin.clone(), vesting_id, lockup_duration);
    LockupToken::issue_wrapped_tokens(env.clone(), minter.clone(), user.clone(), vesting_id, amount);
    
    // Advance time past lockup period
    env.ledger().set_timestamp(env.ledger().timestamp() + lockup_duration + 1);
    
    // Try to unwrap more than available
    LockupToken::unwrap_tokens(env.clone(), user.clone(), vesting_id, unwrap_amount);
}

#[test]
fn test_is_unlocked() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    let minter = Address::random(&env);
    let user = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    LockupToken::add_authorized_minter(env.clone(), admin.clone(), minter.clone());
    
    let vesting_id = 1u32;
    let lockup_duration = 86400u64; // 1 day
    let amount = 1000i128;
    
    LockupToken::configure_lockup(env.clone(), admin.clone(), vesting_id, lockup_duration);
    LockupToken::issue_wrapped_tokens(env.clone(), minter.clone(), user.clone(), vesting_id, amount);
    
    // Should be locked initially
    assert!(!LockupToken::is_unlocked(env.clone(), user.clone(), vesting_id));
    
    // Advance time past lockup period
    env.ledger().set_timestamp(env.ledger().timestamp() + lockup_duration + 1);
    
    // Should be unlocked now
    assert!(LockupToken::is_unlocked(env.clone(), user.clone(), vesting_id));
}

#[test]
fn test_multiple_vesting_ids() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    let minter = Address::random(&env);
    let user = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    LockupToken::add_authorized_minter(env.clone(), admin.clone(), minter.clone());
    
    let vesting_id_1 = 1u32;
    let vesting_id_2 = 2u32;
    let lockup_duration_1 = 86400u64; // 1 day
    let lockup_duration_2 = 172800u64; // 2 days
    let amount_1 = 1000i128;
    let amount_2 = 2000i128;
    
    // Configure different lockup periods for different vesting IDs
    LockupToken::configure_lockup(env.clone(), admin.clone(), vesting_id_1, lockup_duration_1);
    LockupToken::configure_lockup(env.clone(), admin.clone(), vesting_id_2, lockup_duration_2);
    
    // Issue tokens for both vesting IDs
    LockupToken::issue_wrapped_tokens(env.clone(), minter.clone(), user.clone(), vesting_id_1, amount_1);
    LockupToken::issue_wrapped_tokens(env.clone(), minter.clone(), user.clone(), vesting_id_2, amount_2);
    
    // Check total wrapped balance
    let total_balance = LockupToken::wrapped_balance(env.clone(), user.clone());
    assert_eq!(total_balance, amount_1 + amount_2);
    
    // Advance time past first lockup period but not second
    env.ledger().set_timestamp(env.ledger().timestamp() + lockup_duration_1 + 1);
    
    // First vesting ID should be unlocked, second should still be locked
    assert!(LockupToken::is_unlocked(env.clone(), user.clone(), vesting_id_1));
    assert!(!LockupToken::is_unlocked(env.clone(), user.clone(), vesting_id_2));
    
    // Unwrap first vesting ID
    LockupToken::unwrap_tokens(env.clone(), user.clone(), vesting_id_1, amount_1);
    
    // Check balance after partial unwrap
    let remaining_balance = LockupToken::wrapped_balance(env.clone(), user.clone());
    assert_eq!(remaining_balance, amount_2);
}

#[test]
fn test_authorized_minter_management() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    let minter = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    
    // Add authorized minter
    LockupToken::add_authorized_minter(env.clone(), admin.clone(), minter.clone());
    assert!(storage::is_authorized_minter(&env, &minter));
    
    // Remove authorized minter
    LockupToken::remove_authorized_minter(env.clone(), admin.clone(), minter.clone());
    assert!(!storage::is_authorized_minter(&env, &minter));
}

#[test]
fn test_unwrap_history() {
    let env = Env::default();
    let admin = Address::random(&env);
    let underlying_token = Address::random(&env);
    let minter = Address::random(&env);
    let user = Address::random(&env);
    
    LockupToken::initialize(env.clone(), admin.clone(), underlying_token);
    LockupToken::add_authorized_minter(env.clone(), admin.clone(), minter.clone());
    
    let vesting_id = 1u32;
    let lockup_duration = 1u64; // 1 second for testing
    let amount = 1000i128;
    
    LockupToken::configure_lockup(env.clone(), admin.clone(), vesting_id, lockup_duration);
    LockupToken::issue_wrapped_tokens(env.clone(), minter.clone(), user.clone(), vesting_id, amount);
    
    // Advance time past lockup period
    env.ledger().set_timestamp(env.ledger().timestamp() + lockup_duration + 1);
    
    // Unwrap tokens
    LockupToken::unwrap_tokens(env.clone(), user.clone(), vesting_id, amount);
    
    // Check unwrap history
    let history = LockupToken::get_unwrap_history(env.clone());
    assert_eq!(history.len(), 1);
    
    let unwrap_event = history.get(0).unwrap();
    assert_eq!(unwrap_event.user, user);
    assert_eq!(unwrap_event.vesting_id, vesting_id);
    assert_eq!(unwrap_event.amount, amount);
}
