use soroban_sdk::{contractimpl, Address, Env, BytesN};
use lockup_token::{LockupToken, storage, types::*};

pub struct LockupExample;

#[contractimpl]
impl LockupExample {
    /// Example demonstrating the complete lock-up period flow
    pub fn demonstrate_lockup_flow(e: Env) {
        // Setup participants
        let admin = Address::random(&e);
        let underlying_token = Address::random(&e);
        let vesting_vault = Address::random(&e);
        let user = Address::random(&e);
        
        // Step 1: Initialize LockupToken contract
        LockupToken::initialize(e.clone(), admin.clone(), underlying_token.clone());
        
        // Step 2: Add VestingVault as authorized minter
        LockupToken::add_authorized_minter(e.clone(), admin.clone(), vesting_vault.clone());
        
        // Step 3: Configure lock-up period for vesting schedule
        let vesting_id = 1u32;
        let lockup_duration = 86400u64; // 1 day in seconds
        let claim_amount = 1000i128;
        
        LockupToken::configure_lockup(e.clone(), admin.clone(), vesting_id, lockup_duration);
        
        // Step 4: Simulate user claiming tokens (normally called by VestingVault)
        let current_time = e.ledger().timestamp();
        
        // Issue wrapped tokens to user
        LockupToken::issue_wrapped_tokens(
            e.clone(), 
            vesting_vault.clone(), 
            user.clone(), 
            vesting_id, 
            claim_amount
        );
        
        // Step 5: Verify wrapped tokens were issued
        let wrapped_balance = LockupToken::wrapped_balance(e.clone(), user.clone());
        assert_eq!(wrapped_balance, claim_amount);
        
        // Step 6: Check lock-up status (should be locked)
        let is_unlocked = LockupToken::is_unlocked(e.clone(), user.clone(), vesting_id);
        assert!(!is_unlocked);
        
        let lockup_info = LockupToken::get_lockup_info(e.clone(), user.clone(), vesting_id)
            .unwrap();
        assert_eq!(lockup_info.amount, claim_amount);
        assert_eq!(lockup_info.unlock_time, current_time + lockup_duration);
        assert!(!lockup_info.is_unwrapped);
        
        // Step 7: Attempt to unwrap during lock-up (should fail)
        // This would panic: LockupToken::unwrap_tokens(e.clone(), user.clone(), vesting_id, claim_amount);
        
        // Step 8: Advance time past lock-up period
        e.ledger().set_timestamp(current_time + lockup_duration + 1);
        
        // Step 9: Verify tokens are now unlocked
        let is_unlocked = LockupToken::is_unlocked(e.clone(), user.clone(), vesting_id);
        assert!(is_unlocked);
        
        // Step 10: Unwrap tokens
        LockupToken::unwrap_tokens(e.clone(), user.clone(), vesting_id, claim_amount);
        
        // Step 11: Verify unwrap completed successfully
        let final_balance = LockupToken::wrapped_balance(e.clone(), user.clone());
        assert_eq!(final_balance, 0);
        
        let final_lockup_info = LockupToken::get_lockup_info(e.clone(), user.clone(), vesting_id)
            .unwrap();
        assert_eq!(final_lockup_info.amount, 0);
        assert!(final_lockup_info.is_unwrapped);
        
        // Step 12: Check unwrap history
        let unwrap_history = LockupToken::get_unwrap_history(e.clone());
        assert_eq!(unwrap_history.len(), 1);
        
        let unwrap_event = unwrap_history.get(0).unwrap();
        assert_eq!(unwrap_event.user, user);
        assert_eq!(unwrap_event.vesting_id, vesting_id);
        assert_eq!(unwrap_event.amount, claim_amount);
    }
    
    /// Example demonstrating multiple vesting schedules with different lock-up periods
    pub fn demonstrate_multiple_vesting_schedules(e: Env) {
        // Setup
        let admin = Address::random(&e);
        let underlying_token = Address::random(&e);
        let vesting_vault = Address::random(&e);
        let user = Address::random(&e);
        
        LockupToken::initialize(e.clone(), admin.clone(), underlying_token.clone());
        LockupToken::add_authorized_minter(e.clone(), admin.clone(), vesting_vault.clone());
        
        // Configure different lock-up periods
        let vesting_id_1 = 1u32;
        let vesting_id_2 = 2u32;
        let lockup_duration_1 = 86400u64;  // 1 day
        let lockup_duration_2 = 172800u64; // 2 days
        let amount_1 = 1000i128;
        let amount_2 = 2000i128;
        
        LockupToken::configure_lockup(e.clone(), admin.clone(), vesting_id_1, lockup_duration_1);
        LockupToken::configure_lockup(e.clone(), admin.clone(), vesting_id_2, lockup_duration_2);
        
        // Issue tokens for both schedules
        LockupToken::issue_wrapped_tokens(e.clone(), vesting_vault.clone(), user.clone(), vesting_id_1, amount_1);
        LockupToken::issue_wrapped_tokens(e.clone(), vesting_vault.clone(), user.clone(), vesting_id_2, amount_2);
        
        // Check total balance
        let total_balance = LockupToken::wrapped_balance(e.clone(), user.clone());
        assert_eq!(total_balance, amount_1 + amount_2);
        
        // Advance time past first lock-up but not second
        let current_time = e.ledger().timestamp();
        e.ledger().set_timestamp(current_time + lockup_duration_1 + 1);
        
        // First schedule should be unlocked, second still locked
        assert!(LockupToken::is_unlocked(e.clone(), user.clone(), vesting_id_1));
        assert!(!LockupToken::is_unlocked(e.clone(), user.clone(), vesting_id_2));
        
        // Unwrap first schedule
        LockupToken::unwrap_tokens(e.clone(), user.clone(), vesting_id_1, amount_1);
        
        // Check remaining balance
        let remaining_balance = LockupToken::wrapped_balance(e.clone(), user.clone());
        assert_eq!(remaining_balance, amount_2);
        
        // Advance time past second lock-up
        e.ledger().set_timestamp(current_time + lockup_duration_2 + 1);
        
        // Now both should be unlocked
        assert!(LockupToken::is_unlocked(e.clone(), user.clone(), vesting_id_1));
        assert!(LockupToken::is_unlocked(e.clone(), user.clone(), vesting_id_2));
        
        // Unwrap second schedule
        LockupToken::unwrap_tokens(e.clone(), user.clone(), vesting_id_2, amount_2);
        
        // Final balance should be zero
        let final_balance = LockupToken::wrapped_balance(e.clone(), user.clone());
        assert_eq!(final_balance, 0);
    }
    
    /// Example demonstrating partial unwrapping
    pub fn demonstrate_partial_unwrap(e: Env) {
        // Setup
        let admin = Address::random(&e);
        let underlying_token = Address::random(&e);
        let vesting_vault = Address::random(&e);
        let user = Address::random(&e);
        
        LockupToken::initialize(e.clone(), admin.clone(), underlying_token.clone());
        LockupToken::add_authorized_minter(e.clone(), admin.clone(), vesting_vault.clone());
        
        let vesting_id = 1u32;
        let lockup_duration = 86400u64;
        let total_amount = 1000i128;
        let partial_unwrap_amount = 300i128;
        
        LockupToken::configure_lockup(e.clone(), admin.clone(), vesting_id, lockup_duration);
        LockupToken::issue_wrapped_tokens(e.clone(), vesting_vault.clone(), user.clone(), vesting_id, total_amount);
        
        // Advance time past lock-up
        let current_time = e.ledger().timestamp();
        e.ledger().set_timestamp(current_time + lockup_duration + 1);
        
        // Partial unwrap
        LockupToken::unwrap_tokens(e.clone(), user.clone(), vesting_id, partial_unwrap_amount);
        
        // Check remaining balance
        let remaining_balance = LockupToken::wrapped_balance(e.clone(), user.clone());
        assert_eq!(remaining_balance, total_amount - partial_unwrap_amount);
        
        // Check lockup info updated correctly
        let lockup_info = LockupToken::get_lockup_info(e.clone(), user.clone(), vesting_id).unwrap();
        assert_eq!(lockup_info.amount, total_amount - partial_unwrap_amount);
        assert!(!lockup_info.is_unwrapped); // Not fully unwrapped yet
        
        // Unwrap remaining amount
        LockupToken::unwrap_tokens(e.clone(), user.clone(), vesting_id, total_amount - partial_unwrap_amount);
        
        // Verify complete unwrap
        let final_balance = LockupToken::wrapped_balance(e.clone(), user.clone());
        assert_eq!(final_balance, 0);
        
        let final_lockup_info = LockupToken::get_lockup_info(e.clone(), user.clone(), vesting_id).unwrap();
        assert_eq!(final_lockup_info.amount, 0);
        assert!(final_lockup_info.is_unwrapped);
    }
}
