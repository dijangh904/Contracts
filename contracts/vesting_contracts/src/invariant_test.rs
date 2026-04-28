#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Ledger as _},
    token, Address, Env, IntoVal, Symbol, Vec,
};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::{
    oracle::{ComparisonOperator, OracleCondition, OracleType, PerformanceCliff},
    VestingContract, VestingContractClient,
};

/// Helper to setup a simple, single-asset vault for tests
fn setup_test_vault(env: &Env, amount: i128, duration: u64) -> (VestingContractClient, u64) {
    let client = VestingContractClient::new(env, &env.register_contract(None, VestingContract));
    let admin = Address::generate(env);
    let token_id = env.register_stellar_asset_contract(Address::generate(env));
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    // Mint tokens to the admin address that will fund the vaults
    token_admin_client.mint(&admin, &(amount + 1000)); // mint extra for fees

    // Initialize the vesting contract
    client.initialize(&admin, &amount);
    client.set_token(&token_id);

    let owner = Address::generate(env);
    let start_time = env.ledger().timestamp();
    let end_time = start_time + duration;

    // create_vault_full will pull funds from the admin address
    let vault_id = client.create_vault_full(
        &owner,
        &amount,
        &start_time,
        &end_time,
        &0,     // keeper_fee
        &true,  // is_revocable
        &false, // is_transferable
        &0,     // step_duration
    );

    (client, vault_id)
}

/// A mock oracle contract for testing performance cliffs.
#[contract]
pub struct MockOracle;

#[contractimpl]
impl MockOracle {
    /// The vesting contract will call this.
    pub fn is_cliff_passed(env: Env, _cliff: PerformanceCliff, _vault_id: u64) -> bool {
        env.storage()
            .instance()
            .get(&symbol_short!("passed"))
            .unwrap_or(false)
    }

    /// Test helper to control the mock's return value.
    pub fn set_passed(env: Env, passed: bool) {
        env.storage().instance().set(&symbol_short!("passed"), &passed);
    }
}

#[test]
fn invariant_total_claims_le_total_allocation() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, vault_id) = setup_test_vault(&env, 1_000_000_000, 1000);

    let mut total_claimed = 0i128;
    let mut rng = ChaCha20Rng::seed_from_u64(1);

    for _ in 0..100 {
        // 100 random claims
        let vault = client.get_vault(&vault_id);
        let total_allocation = vault.allocations.get(0).unwrap().total_amount;

        // Advance time randomly
        let current_time = env.ledger().timestamp();
        let new_time = rng.gen_range(current_time..=vault.end_time + 100);
        env.ledger().with_mut(|l| l.timestamp = new_time);

        let claimable = client.get_claimable_amount(&vault_id);
        if claimable > 0 {
            let claim_amount = rng.gen_range(1..=claimable);
            client.claim_tokens(&vault_id, &claim_amount);
            total_claimed += claim_amount;
        }

        // Invariant check
        let updated_vault = client.get_vault(&vault_id);
        let released_amount = updated_vault.allocations.get(0).unwrap().released_amount;
        assert!(released_amount <= total_allocation);
        assert_eq!(released_amount, total_claimed);
    }

    // Final check
    let final_vault = client.get_vault(&vault_id);
    let final_released = final_vault.allocations.get(0).unwrap().released_amount;
    let total_allocation = final_vault.allocations.get(0).unwrap().total_amount;
    assert!(final_released <= total_allocation);
    assert_eq!(final_released, total_claimed);
}

#[test]
fn invariant_cliff_boundaries_are_absolute() {
    let env = Env::default();
    env.mock_all_auths();

    // --- Time-based cliff ---
    let (client, vault_id) = setup_test_vault(&env, 1_000_000_000, 1000);
    let vault = client.get_vault(&vault_id);
    let start_time = vault.start_time;

    env.ledger().with_mut(|l| l.timestamp = 0);
    assert_eq!(client.get_claimable_amount(&vault_id), 0);

    env.ledger().with_mut(|l| l.timestamp = start_time - 1);
    assert_eq!(client.get_claimable_amount(&vault_id), 0);

    env.ledger().with_mut(|l| l.timestamp = start_time);
    assert_eq!(client.get_claimable_amount(&vault_id), 0);

    env.ledger().with_mut(|l| l.timestamp = start_time + 1);
    assert!(client.get_claimable_amount(&vault_id) > 0);

    // --- Performance-based cliff ---
    let oracle_id = env.register_contract(None, MockOracle);
    let cliff = PerformanceCliff {
        oracle: oracle_id.clone(),
        conditions: Vec::new(&env), // Conditions are unused by this mock
    };
    client.set_performance_cliff(&vault_id, &cliff);

    // Mock oracle to return `false` for cliff passed
    env.invoke_contract::<()>(&oracle_id, &symbol_short!("set_passed"), (false,).into_val(&env));

    // Advance time well past the start, but cliff is not met
    env.ledger().with_mut(|l| l.timestamp = start_time + 500);
    assert!(!client.is_cliff_passed(&vault_id));
    assert_eq!(client.get_claimable_amount(&vault_id), 0);

    // Mock oracle to return `true`
    env.invoke_contract::<()>(&oracle_id, &symbol_short!("set_passed"), (true,).into_val(&env));
    assert!(client.is_cliff_passed(&vault_id));
    assert!(client.get_claimable_amount(&vault_id) > 0);
}

#[test]
fn fuzz_timestamp_manipulation_resilience() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, vault_id) = setup_test_vault(&env, 1_000_000_000, 315_360_000); // 10 years
    let vault = client.get_vault(&vault_id);
    let start = vault.start_time;
    let end = vault.end_time;
    let total_amount = vault.allocations.get(0).unwrap().total_amount;
    let duration = end - start;

    let mut rng = ChaCha20Rng::seed_from_u64(2);

    for _ in 0..200 {
        // 200 random time jumps
        let random_time = rng.gen_range(start.saturating_sub(1000)..=end.saturating_add(1000));
        env.ledger().with_mut(|l| l.timestamp = random_time);

        let expected_vested = if random_time <= start {
            0
        } else if random_time >= end {
            total_amount
        } else {
            (total_amount * (random_time - start) as i128) / (duration as i128)
        };

        let contract_vested = client.calculate_claimable_for_asset_wrapper(&vault_id, &0);
        assert_eq!(contract_vested, expected_vested);
    }
}

#[test]
fn fuzz_stroop_rounding_error_accumulation() {
    let env = Env::default();
    env.mock_all_auths();

    let total_allocation = 100_000_000 * 10_000_000; // 100M tokens with 7 decimals
    let (client, vault_id) = setup_test_vault(&env, total_allocation, 1000);
    let vault = client.get_vault(&vault_id);

    // Fast-forward to fully vest
    env.ledger().with_mut(|l| l.timestamp = vault.end_time + 1);

    let num_micro_claims = 10_000;

    // Perform 10,000 micro-claims of 1 stroop
    for _ in 0..num_micro_claims {
        client.claim_tokens(&vault_id, &1);
    }

    // Claim the remainder
    let remaining_claimable = client.get_claimable_amount(&vault_id);
    if remaining_claimable > 0 {
        client.claim_tokens(&vault_id, &remaining_claimable);
    }

    // Final invariant check: total claimed must equal total allocation
    let final_vault = client.get_vault(&vault_id);
    let final_released = final_vault.allocations.get(0).unwrap().released_amount;

    assert_eq!(final_released, total_allocation);
    assert_eq!(client.get_claimable_amount(&vault_id), 0);
}