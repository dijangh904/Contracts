use soroban_sdk::{Env, Address, Map, Vec};
use crate::types::{LockupConfig, LockupInfo, UnwrapEvent};

// Storage keys
pub const ADMIN: &str = "ADMIN";
pub const UNDERLYING_TOKEN: &str = "UNDERLYING_TOKEN";
pub const LOCKUP_CONFIGS: &str = "LOCKUP_CONFIGS";
pub const WRAPPED_BALANCES: &str = "WRAPPED_BALANCES";
pub const LOCKUP_INFOS: &str = "LOCKUP_INFOS";
pub const UNWRAP_HISTORY: &str = "UNWRAP_HISTORY";
pub const AUTHORIZED_MINTERS: &str = "AUTHORIZED_MINTERS";

// Admin functions
pub fn has_admin(e: &Env) -> bool {
    e.storage().instance().has_key(&ADMIN)
}

pub fn get_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&ADMIN)
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&ADMIN, admin);
}

// Underlying token functions
pub fn get_underlying_token(e: &Env) -> Option<Address> {
    e.storage().instance().get(&UNDERLYING_TOKEN)
}

pub fn set_underlying_token(e: &Env, token: &Address) {
    e.storage().instance().set(&UNDERLYING_TOKEN, token);
}

// Lockup configuration functions
pub fn get_lockup_config(e: &Env, vesting_id: u32) -> Option<LockupConfig> {
    e.storage().instance().get(&(LOCKUP_CONFIGS, vesting_id))
}

pub fn set_lockup_config(e: &Env, vesting_id: u32, config: &LockupConfig) {
    e.storage().instance().set(&(LOCKUP_CONFIGS, vesting_id), config);
}

// Wrapped balance functions
pub fn get_wrapped_balance(e: &Env, user: &Address) -> i128 {
    e.storage()
        .instance()
        .get(&(WRAPPED_BALANCES, user))
        .unwrap_or(0)
}

pub fn set_wrapped_balance(e: &Env, user: &Address, balance: i128) {
    if balance == 0 {
        e.storage().instance().remove(&(WRAPPED_BALANCES, user));
    } else {
        e.storage().instance().set(&(WRAPPED_BALANCES, user), &balance);
    }
}

// Lockup info functions
pub fn get_lockup_info(e: &Env, user: &Address, vesting_id: u32) -> Option<LockupInfo> {
    e.storage().instance().get(&(LOCKUP_INFOS, user, vesting_id))
}

pub fn set_lockup_info(e: &Env, user: &Address, vesting_id: u32, info: &LockupInfo) {
    e.storage().instance().set(&(LOCKUP_INFOS, user, vesting_id), info);
}

// Unwrap history functions
pub fn get_unwrap_history(e: &Env) -> Vec<UnwrapEvent> {
    e.storage()
        .instance()
        .get(&UNWRAP_HISTORY)
        .unwrap_or(Vec::new(e))
}

pub fn add_unwrap_event(e: &Env, event: &UnwrapEvent) {
    let mut history = get_unwrap_history(e);
    history.push_back(event.clone());
    e.storage().instance().set(&UNWRAP_HISTORY, &history);
}

// Authorized minter functions
pub fn is_authorized_minter(e: &Env, minter: &Address) -> bool {
    e.storage()
        .instance()
        .get(&(AUTHORIZED_MINTERS, minter))
        .unwrap_or(false)
}

pub fn add_authorized_minter(e: &Env, minter: &Address) {
    e.storage().instance().set(&(AUTHORIZED_MINTERS, minter), &true);
}

pub fn remove_authorized_minter(e: &Env, minter: &Address) {
    e.storage().instance().remove(&(AUTHORIZED_MINTERS, minter));
}

pub fn get_authorized_minters(e: &Env) -> Vec<Address> {
    // This would require iterating through all authorized minters
    // For simplicity, we'll return an empty vector for now
    // In a production implementation, you might maintain a separate list
    Vec::new(e)
}
