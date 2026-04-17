#![cfg(test)]

use crate::{
    BatchCreateData, Milestone, PausedVault, VestingContract, VestingContractClient,
    GovernanceAction, GovernanceProposal, Vote, StakeState, ScheduleConfig, AssetAllocationEntry,
    SuccessionState, NominatedData, ClaimPendingData, SucceededData,
    MIN_SWITCH_DURATION, MAX_SWITCH_DURATION, MIN_CHALLENGE_WINDOW, MAX_CHALLENGE_WINDOW,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, vec, Address, Env, String, IntoVal, Symbol, Map,
};

fn setup() -> (Env, Address, VestingContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &1_000_000_000i128);

    let token_admin = Address::generate(&env);
    let token_addr = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
    client.set_token(&token_addr);
    client.add_to_whitelist(&token_addr);

    // Mint initial supply to contract
    let stellar = token::StellarAssetClient::new(&env, &token_addr);
    stellar.mint(&contract_id, &1_000_000_000i128);

    (env, contract_id, client, admin, token_addr)
}

#[test]
fn test_initialize() {
    let (env, _, client, admin, _) = setup();
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_create_vault_full_and_claim() {
    let (env, _, client, _admin, token) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();

    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false, // is_revocable = false
        &false, // is_transferable = false
        &0u64   // step_duration
    );

    assert_eq!(vault_id, 1);

    // Fast forward
    env.ledger().set_timestamp(now + 500);
    assert_eq!(client.get_claimable_amount(&vault_id), 500);

    // Claim
    client.claim_tokens(&vault_id, &100i128);
    assert_eq!(client.get_claimable_amount(&vault_id), 400);

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&beneficiary), 100);
}

#[test]
fn test_periodic_vesting() {
    let (env, _, client, _, _) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();

    // 1000 tokens over 1000 seconds, with 100 second steps
    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &true,
        &false,
        &100u64
    );

    env.ledger().set_timestamp(now + 150);
    // 1 step completed (100 tokens)
    assert_eq!(client.get_claimable_amount(&vault_id), 100);

    env.ledger().set_timestamp(now + 250);
    // 2 steps completed (200 tokens)
    assert_eq!(client.get_claimable_amount(&vault_id), 200);
}

#[test]
fn test_milestones() {
    let (env, _, client, _admin, _) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();

    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &true,
        &false,
        &0u64
    );

    let milestones = vec![
        &env,
        Milestone { id: 1, percentage: 30, is_unlocked: false },
        Milestone { id: 2, percentage: 70, is_unlocked: false }
    ];

    client.set_milestones(&vault_id, &milestones);

    assert_eq!(client.get_claimable_amount(&vault_id), 0);

    client.unlock_milestone(&vault_id, &1);
    assert_eq!(client.get_claimable_amount(&vault_id), 300);

    client.unlock_milestone(&vault_id, &2);
    assert_eq!(client.get_claimable_amount(&vault_id), 1000);
}

#[test]
fn test_global_pause() {
    let (env, _, client, _, _) = setup();

    client.toggle_pause();
    assert!(client.is_paused());

    client.toggle_pause();
    assert!(!client.is_paused());
}

#[test]
fn test_batch_operations() {
    let (env, _, client, _, _) = setup();
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let now = env.ledger().timestamp();

    let batch = BatchCreateData {
        recipients: vec![&env, r1, r2],
        amounts: vec![&env, 500i128, 500i128],
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
    let (env, _, client, _, token) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();
    
    // Create an irrevocable vault (100% weight)
    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &false,
        &0u64
    );
    
    assert_eq!(client.get_voter_power(&beneficiary), 1000);
    
    // Create a revocable vault (50% weight)
    client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &true,
        &false,
        &0u64
    );
    
    // Total should be 1000 + 500 = 1500
    assert_eq!(client.get_voter_power(&beneficiary), 1500);
}

#[test]
fn test_pause_specific_schedule() {
    let (env, _, client, _admin, _) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();

    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &false,
        &0u64
    );

    // Fast forward to allow some vesting
    env.ledger().set_timestamp(now + 500);
    assert_eq!(client.get_claimable_amount(&vault_id), 500);

    // Pause the vault
    client.pause_specific_schedule(&vault_id, &String::from_str(&env, "Legal dispute"));

    // Check that vault is paused
    assert!(client.is_vault_paused(&vault_id));

    // Get pause info
    let pause_info = client.get_paused_vault_info(&vault_id).unwrap();
    assert_eq!(pause_info.vault_id, vault_id);
    assert_eq!(pause_info.pause_timestamp, now + 500);
    assert_eq!(pause_info.reason, String::from_str(&env, "Legal dispute"));
}

#[test]
fn test_slashing() {
    let (env, _, client, _, token) = setup();
    let beneficiary = Address::generate(&env);
    let treasury = Address::generate(&env);
    let token_client = token::Client::new(&env, &token);
    let now = env.ledger().timestamp();

    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &false,
        &0u64
    );

    env.ledger().set_timestamp(now + 400);
    assert_eq!(client.get_claimable_amount(&vault_id), 400);

    client.slash_unvested_balance(&vault_id, &treasury);

    // 600 unvested tokens slashed to treasury
    assert_eq!(token_client.balance(&treasury), 600);
    assert_eq!(client.get_claimable_amount(&vault_id), 400);

    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.total_amount, 400);

    env.ledger().set_timestamp(now + 1000);
    assert_eq!(client.get_claimable_amount(&vault_id), 400); // capped at 400
}

#[test]
fn test_metadata_anchor() {
    let (env, _, client, _, _) = setup();

    // Should return empty string before anything is set
    let empty = client.get_metadata_anchor();
    assert_eq!(empty, String::from_str(&env, ""));

    // Set a CID and retrieve it
    let cid = String::from_str(&env, "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi");
    client.set_metadata_anchor(&cid);

    let retrieved = client.get_metadata_anchor();
    assert_eq!(retrieved, cid);
}

#[test]
fn test_delegated_voting_power() {
    let (env, _, client, _, _) = setup();
    let beneficiary_a = Address::generate(&env);
    let beneficiary_b = Address::generate(&env);
    let representative = Address::generate(&env);
    let now = env.ledger().timestamp();

    // A: 1000 power (irrevocable)
    client.create_vault_full(&beneficiary_a, &1000i128, &now, &(now + 1000), &0i128, &false, &false, &0u64);

    // B: 500 power (revocable)
    client.create_vault_full(&beneficiary_b, &1000i128, &now, &(now + 1000), &0i128, &true, &false, &0u64);

    assert_eq!(client.get_voter_power(&beneficiary_a), 1000);
    assert_eq!(client.get_voter_power(&beneficiary_b), 500);

    client.delegate_voting_power(&beneficiary_a, &beneficiary_b);
    assert_eq!(client.get_voter_power(&beneficiary_a), 0);
    assert_eq!(client.get_voter_power(&beneficiary_b), 1500);

    client.delegate_voting_power(&beneficiary_b, &representative);
    assert_eq!(client.get_voter_power(&beneficiary_b), 0);
    assert_eq!(client.get_voter_power(&representative), 1500);
}

#[test]
fn test_nominate_valid_backup_state_is_nominated() {
    let (env, _, client, _, _) = setup();
    let primary = Address::generate(&env);
    let backup = Address::generate(&env);
    let now = env.ledger().timestamp();

    let vault_id = client.create_vault_full(&primary, &1000i128, &now, &(now + 1000), &0i128, &true, &false, &0u64);

    client.nominate_backup(&vault_id, &backup, &MIN_SWITCH_DURATION, &MIN_CHALLENGE_WINDOW);

    let view = client.get_succession_status(&vault_id);
    assert_eq!(view.backup, Some(backup));
    assert!(matches!(view.state, SuccessionState::Nominated(_)));
}

#[test]
fn test_marketplace_listing_and_sale() {
    let (env, _, client, _admin, _) = setup();
    let beneficiary = Address::generate(&env);
    let marketplace = Address::generate(&env);
    let buyer = Address::generate(&env);
    let now = env.ledger().timestamp();
    
    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &true, // must be transferable
        &0u64
    );
    
    // 1. Beneficiary authorizes marketplace
    client.authorize_transfer_to_marketplace(&vault_id, &marketplace);
    
    // 2. Marketplace completes transfer to buyer
    client.complete_marketplace_transfer(&vault_id, &buyer);
    
    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.owner, buyer);
}

#[test]
fn test_vault_renewal() {
    let (env, _, client, admin, _) = setup();
    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();
    
    let vault_id = client.create_vault_full(
        &beneficiary,
        &1000i128,
        &now,
        &(now + 1000),
        &0i128,
        &false,
        &false,
        &0u64
    );
    
    // Renew: add 1000 seconds and 500 tokens
    client.renew_schedule(&vault_id, &1000u64, &500i128);
    
    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.end_time, now + 2000);
    assert_eq!(vault.allocations.get(0).unwrap().total_amount, 1500);
}
