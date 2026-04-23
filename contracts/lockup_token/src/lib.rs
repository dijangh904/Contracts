#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Map, Vec, String};

mod storage;
pub mod types;

use types::*;
use storage::{get_lockup_config, set_lockup_config, get_wrapped_balance, set_wrapped_balance, get_lockup_info, set_lockup_info, get_unwrap_history, add_unwrap_event};

#[contract]
pub struct LockupToken;

#[contractimpl]
impl LockupToken {
    /// Initialize the lockup token contract
    pub fn initialize(e: Env, admin: Address, underlying_token: Address) {
        admin.require_auth();
        
        if storage::has_admin(&e) {
            panic!("Contract already initialized");
        }
        
        storage::set_admin(&e, &admin);
        storage::set_underlying_token(&e, &underlying_token);
        
        Initialized {
            admin: admin.clone(),
            underlying_token,
            timestamp: e.ledger().timestamp(),
        }.publish(&e);
    }

    /// Configure lockup period for a specific vesting schedule
    pub fn configure_lockup(e: Env, admin: Address, vesting_id: u32, lockup_duration_seconds: u64) {
        admin.require_auth();
        
        Self::require_admin(&e);
        
        let config = LockupConfig {
            vesting_id,
            lockup_duration_seconds,
            created_at: e.ledger().timestamp(),
        };
        
        set_lockup_config(&e, vesting_id, &config);
        
        LockupConfigured {
            vesting_id,
            lockup_duration_seconds,
            timestamp: e.ledger().timestamp(),
        }.publish(&e);
    }

    /// Issue wrapped tokens to a user (called by vesting vault during claim)
    pub fn issue_wrapped_tokens(e: Env, from: Address, to: Address, vesting_id: u32, amount: i128) {
        from.require_auth();
        
        // Verify caller is authorized (should be vesting vault)
        Self::require_authorized_minter(&e, &from);
        
        // Get lockup configuration
        let config = get_lockup_config(&e, vesting_id)
            .expect("No lockup configuration found for vesting ID");
        
        let current_time = e.ledger().timestamp();
        let unlock_time = current_time + config.lockup_duration_seconds;
        
        // Create lockup info
        let lockup_info = LockupInfo {
            vesting_id,
            amount,
            locked_at: current_time,
            unlock_time,
            is_unwrapped: false,
        };
        
        // Store lockup info
        set_lockup_info(&e, &to, vesting_id, &lockup_info);
        
        // Update wrapped balance
        let current_balance = get_wrapped_balance(&e, &to);
        set_wrapped_balance(&e, &to, current_balance + amount);
        
        WrappedTokensIssued {
            to: to.clone(),
            vesting_id,
            amount,
            unlock_time,
            timestamp: current_time,
        }.publish(&e);
    }

    /// Unwrap tokens after lockup period expires
    pub fn unwrap_tokens(e: Env, user: Address, vesting_id: u32, amount: i128) {
        user.require_auth();
        
        let current_time = e.ledger().timestamp();
        
        // Get lockup info
        let mut lockup_info = get_lockup_info(&e, &user, vesting_id)
            .expect("No lockup info found for user and vesting ID");
        
        // Check if lockup period has expired
        if current_time < lockup_info.unlock_time {
            panic!("Tokens are still locked until {}", lockup_info.unlock_time);
        }
        
        // Check if user has sufficient wrapped balance
        let wrapped_balance = get_wrapped_balance(&e, &user);
        if wrapped_balance < amount {
            panic!("Insufficient wrapped token balance");
        }
        
        // Check if trying to unwrap more than locked amount
        if amount > lockup_info.amount {
            panic!("Cannot unwrap more than locked amount");
        }
        
        // Update wrapped balance
        set_wrapped_balance(&e, &user, wrapped_balance - amount);
        
        // Update lockup info
        lockup_info.amount -= amount;
        if lockup_info.amount == 0 {
            lockup_info.is_unwrapped = true;
        }
        set_lockup_info(&e, &user, vesting_id, &lockup_info);
        
        // Add to unwrap history
        let unwrap_event = UnwrapEvent {
            user: user.clone(),
            vesting_id,
            amount,
            timestamp: current_time,
        };
        add_unwrap_event(&e, &unwrap_event);
        
        // In a real implementation, this would transfer the underlying tokens
        // For now, we'll just emit the event
        TokensUnwrapped {
            user: user.clone(),
            vesting_id,
            amount,
            timestamp: current_time,
        }.publish(&e);
    }

    /// Get wrapped token balance for a user
    pub fn wrapped_balance(e: Env, user: Address) -> i128 {
        get_wrapped_balance(&e, &user)
    }

    /// Get lockup info for a user and vesting ID
    pub fn get_lockup_info(e: Env, user: Address, vesting_id: u32) -> Option<LockupInfo> {
        get_lockup_info(&e, &user, vesting_id)
    }

    /// Get lockup configuration for a vesting ID
    pub fn get_lockup_config(e: Env, vesting_id: u32) -> Option<LockupConfig> {
        get_lockup_config(&e, vesting_id)
    }

    /// Check if tokens are unlocked for a user and vesting ID
    pub fn is_unlocked(e: Env, user: Address, vesting_id: u32) -> bool {
        if let Some(lockup_info) = get_lockup_info(&e, &user, vesting_id) {
            let current_time = e.ledger().timestamp();
            current_time >= lockup_info.unlock_time
        } else {
            false
        }
    }

    /// Get unwrap history
    pub fn get_unwrap_history(e: Env) -> Vec<UnwrapEvent> {
        get_unwrap_history(&e)
    }

    /// Add an authorized minter (typically the vesting vault)
    pub fn add_authorized_minter(e: Env, admin: Address, minter: Address) {
        admin.require_auth();
        Self::require_admin(&e);
        
        storage::add_authorized_minter(&e, &minter);
        
        AuthorizedMinterAdded {
            minter: minter.clone(),
            timestamp: e.ledger().timestamp(),
        }.publish(&e);
    }

    /// Remove an authorized minter
    pub fn remove_authorized_minter(e: Env, admin: Address, minter: Address) {
        admin.require_auth();
        Self::require_admin(&e);
        
        storage::remove_authorized_minter(&e, &minter);
        
        AuthorizedMinterRemoved {
            minter: minter.clone(),
            timestamp: e.ledger().timestamp(),
        }.publish(&e);
    }

    /// Internal helper to require admin authorization
    fn require_admin(e: &Env) {
        let admin = storage::get_admin(e).expect("Admin not set");
        admin.require_auth();
    }

    /// Internal helper to require authorized minter
    fn require_authorized_minter(e: &Env, minter: &Address) {
        if !storage::is_authorized_minter(e, minter) {
            panic!("Not an authorized minter");
        }
    }
}
