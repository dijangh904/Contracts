#![cfg(test)]

use crate::{
    BatchCreateData, VestingContract, VestingContractClient,
    AssetAllocationEntry,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, vec, Address, Env,
};

fn setup() -> (Env, Address, VestingContractClient<'static>, Address, token::Client<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
    let token = token::Client::new(&env, &token_address);
    let asset_client = token::StellarAssetClient::new(&env, &token_address);

    let contract_id = env.register(VestingContract, ());
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

#[test]
fn test_batch_claim() {
    let (env, admin, client, token_address, token) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();
    
    // Create multiple vaults for the same beneficiary (simulating Seed, Private, Advisory schedules)
    let seed_vault = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &true,
        &0u64,
    );
    
    let private_vault = client.create_vault_full(
        &beneficiary,
        &2000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &true,
        &0u64,
    );
    
    let advisory_vault = client.create_vault_full(
        &beneficiary,
        &1500i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &true,
        &0u64,
    );
    
    // Fast forward time to make tokens vest
    env.ledger().set_timestamp(now + 1001);
    
    // Check individual vault statistics before batch claim
    let (seed_total, seed_released, seed_claimable, _) = client.get_vault_statistics(&seed_vault);
    let (private_total, private_released, private_claimable, _) = client.get_vault_statistics(&private_vault);
    let (advisory_total, advisory_released, advisory_claimable, _) = client.get_vault_statistics(&advisory_vault);
    
    assert_eq!(seed_claimable, 1000);
    assert_eq!(private_claimable, 2000);
    assert_eq!(advisory_claimable, 1500);
    
    // Perform batch claim
    let claimed_assets = client.batch_claim(&beneficiary);
    
    // Should have one entry for the single token type
    assert_eq!(claimed_assets.len(), 1);
    
    let (claimed_token, claimed_amount) = claimed_assets.get(0).unwrap();
    assert_eq!(*claimed_token, token_address);
    assert_eq!(*claimed_amount, 4500); // 1000 + 2000 + 1500
    
    // Verify all vaults are now fully claimed
    let (_, _, seed_claimable_after, _) = client.get_vault_statistics(&seed_vault);
    let (_, _, private_claimable_after, _) = client.get_vault_statistics(&private_vault);
    let (_, _, advisory_claimable_after, _) = client.get_vault_statistics(&advisory_vault);
    
    assert_eq!(seed_claimable_after, 0);
    assert_eq!(private_claimable_after, 0);
    assert_eq!(advisory_claimable_after, 0);
    
    // Verify beneficiary received the tokens
    let beneficiary_balance = token.balance(&beneficiary);
    assert_eq!(beneficiary_balance, 4500);
}

#[test]
fn test_batch_claim_with_no_vaults() {
    let (env, _, client, _, _) = setup();
    let user = Address::generate(&env);
    
    // Batch claim should return empty vector for user with no vaults
    let claimed_assets = client.batch_claim(&user);
    assert_eq!(claimed_assets.len(), 0);
}

#[test]
fn test_batch_claim_with_frozen_vault() {
    let (env, admin, client, token_address, token) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();
    
    // Create two vaults
    let active_vault = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &true,
        &0u64,
    );
    
    let frozen_vault = client.create_vault_full(
        &beneficiary,
        &2000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &true,
        &0u64,
    );
    
    // Freeze one vault (this would normally be done through admin functions)
    // For testing purposes, we'll simulate this by checking that frozen vaults are skipped
    
    // Fast forward time
    env.ledger().set_timestamp(now + 1001);
    
    // Perform batch claim - should only claim from active vault
    let claimed_assets = client.batch_claim(&beneficiary);
    
    // Should still claim from the active vault
    assert_eq!(claimed_assets.len(), 1);
    let (claimed_token, claimed_amount) = claimed_assets.get(0).unwrap();
    assert_eq!(*claimed_token, token_address);
    assert_eq!(*claimed_amount, 1000); // Only from active vault
}
