#![cfg(test)]

use crate::{
    BatchCreateData, Milestone, PausedVault, VestingContract, VestingContractClient,
    AssetAllocationEntry,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, vec, Address, Env, String, IntoVal,
};

fn setup() -> (Env, Address, VestingContractClient<'static>, Address, token::Client<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract(token_admin.clone());
    let token = token::Client::new(&env, &token_address);
    let asset_client = token::StellarAssetClient::new(&env, &token_address);

    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);

    client.initialize(&admin, &1_000_000_000i128);
    client.set_token(&token_address);
    
    // Prefund sub-admin balance for testing
    asset_client.mint(&admin, &1_000_000);

    (env, admin, client, token_address, token)
}

#[test]
fn test_initialize() {
    let (_env, admin, client, _token_address, _) = setup();
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_create_vault() {
    let (env, _, client, _, _) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();

    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &true,
        &true,
        &0u64,
    );

    assert_eq!(vault_id, 1);
    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.owner, beneficiary);
    assert_eq!(vault.allocations.get(0).unwrap().total_amount, 1000);
}

#[test]
fn test_claim_tokens() {
    let (env, _, client, _, token) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();

    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &true,
        &true,
        &0u64,
    );

    // Fast forward halfway
    env.ledger().with_mut(|li| {
        li.timestamp = now + 500;
    });

    client.claim_tokens(&vault_id, &500i128);
    assert_eq!(token.balance(&beneficiary), 500);
}

#[test]
fn test_revoke_vault() {
    let (env, admin, client, _, _) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();

    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &true,
        &true,
        &0u64,
    );

    client.revoke_vault(&vault_id, &admin);
    let vault = client.get_vault(&vault_id);
    assert!(vault.is_frozen);
}

#[test]
fn test_pause_resume() {
    let (_env, _admin, client, _, _) = setup();
    
    client.pause();
    assert!(client.is_paused());
    
    client.resume();
    assert!(!client.is_paused());
}

#[test]
fn test_batch_operations() {
    let (env, _, client, token_address, _) = setup();
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let now = env.ledger().timestamp();

    let allocation1 = AssetAllocationEntry {
        asset_id: token_address.clone(),
        total_amount: 500,
        released_amount: 0,
        locked_amount: 0,
        percentage: 10000,
    };
    let mut basket1 = vec![&env];
    basket1.push_back(allocation1);

    let allocation2 = AssetAllocationEntry {
        asset_id: token_address.clone(),
        total_amount: 500,
        released_amount: 0,
        locked_amount: 0,
        percentage: 10000,
    };
    let mut basket2 = vec![&env];
    basket2.push_back(allocation2);

    let batch = BatchCreateData {
        recipients: vec![&env, r1, r2],
        asset_baskets: vec![&env, basket1, basket2],
        start_times: vec![&env, now, now],
        end_times: vec![&env, now + 1000, now + 1000],
        keeper_fees: vec![&env, 0i128, 0i128],
        step_durations: vec![&env, 0u64, 0u64],
    };

    let ids = client.batch_create_vaults_full(&batch);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), 1);
    assert_eq!(ids.get(1).unwrap(), 2);
}

#[test]
fn test_voting_power_calculation() {
    let (env, _, client, _, _) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();
    
    let _vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &true,
        &true,
        &0u64,
    );
    
    let voting_power = client.get_voting_power(&beneficiary);
    assert!(voting_power > 0);
}

#[test]
fn test_marketplace_transfer() {
    let (env, _, client, _, _) = setup();
    let beneficiary = Address::generate(&env);
    let marketplace = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let now = env.ledger().timestamp();
    
    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &true,
        &true, // must be transferable
        &0u64,
    );
    
    // Authorize transfer
    client.authorize_marketplace_transfer(&vault_id, &marketplace);
    
    // Complete transfer
    client.complete_marketplace_transfer(&vault_id, &new_owner);
    
    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.owner, new_owner);
}
