#![cfg(test)]

use soroban_sdk::{Address, Env};
use vesting_vault::{VestingVault, Error, LSTConfig, LSTPoolShares, UserLSTShares, UnbondingRequest};

#[test]
fn test_lst_compounding_configuration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64; // 7 days

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Verify configuration was stored
    let config = VestingVault::get_lst_config(env.clone(), vesting_id).unwrap();
    assert!(config.enabled);
    assert_eq!(config.vesting_id, vesting_id);
    assert_eq!(config.lst_token_address, lst_token);
    assert_eq!(config.base_token_address, base_token);
    assert_eq!(config.staking_contract_address, staking_contract);
    assert_eq!(config.unbonding_period_seconds, unbonding_period);
}

#[test]
fn test_deposit_to_lst_pool() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;
    let deposit_amount = 1000i128;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Deposit tokens to pool
    VestingVault::deposit_to_lst_pool(
        env.clone(),
        user.clone(),
        vesting_id,
        deposit_amount,
    )
    .unwrap();

    // Verify pool state
    let pool_shares = VestingVault::get_lst_pool_shares(env.clone(), vesting_id).unwrap();
    assert_eq!(pool_shares.total_shares, deposit_amount);
    assert_eq!(pool_shares.total_underlying, deposit_amount);

    // Verify user shares
    let user_shares = VestingVault::get_user_lst_shares(env.clone(), &user, vesting_id).unwrap();
    assert_eq!(user_shares.shares, deposit_amount);
    assert!(!user_shares.unbonding_pending);
}

#[test]
fn test_lst_compounding_rewards() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;
    let deposit_amount = 1000i128;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Deposit tokens
    VestingVault::deposit_to_lst_pool(
        env.clone(),
        user.clone(),
        vesting_id,
        deposit_amount,
    )
    .unwrap();

    // Advance time to allow compounding (more than 1 hour)
    env.ledger().set_timestamp(env.ledger().timestamp() + 7200);

    // Compound rewards (will return Ok with no rewards in simulation)
    let result = VestingVault::compound_lst_rewards(env.clone(), vesting_id);
    assert!(result.is_ok());
}

#[test]
fn test_exchange_rate_manipulation_protection() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;
    let deposit_amount = 1000i128;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Deposit tokens
    VestingVault::deposit_to_lst_pool(
        env.clone(),
        user.clone(),
        vesting_id,
        deposit_amount,
    )
    .unwrap();

    // Try to compound immediately (less than 1 hour since snapshot)
    let result = VestingVault::compound_lst_rewards(env.clone(), vesting_id);
    assert!(matches!(result, Err(Error::ExchangeRateManipulationSuspected)));
}

#[test]
fn test_unbonding_request() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;
    let deposit_amount = 1000i128;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Deposit tokens
    VestingVault::deposit_to_lst_pool(
        env.clone(),
        user.clone(),
        vesting_id,
        deposit_amount,
    )
    .unwrap();

    // Request unbonding
    VestingVault::request_unbonding(env.clone(), user.clone(), vesting_id).unwrap();

    // Verify unbonding request was created
    let unbonding_request = VestingVault::get_unbonding_request(env.clone(), &user, vesting_id).unwrap();
    assert_eq!(unbonding_request.shares, deposit_amount);
    assert!(unbonding_request.unbonding_complete_at > env.ledger().timestamp());

    // Verify user shares marked as pending
    let user_shares = VestingVault::get_user_lst_shares(env.clone(), &user, vesting_id).unwrap();
    assert!(user_shares.unbonding_pending);
}

#[test]
fn test_unbonding_period_not_elapsed() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;
    let deposit_amount = 1000i128;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Deposit tokens
    VestingVault::deposit_to_lst_pool(
        env.clone(),
        user.clone(),
        vesting_id,
        deposit_amount,
    )
    .unwrap();

    // Request unbonding
    VestingVault::request_unbonding(env.clone(), user.clone(), vesting_id).unwrap();

    // Try to complete unbonding immediately (period not elapsed)
    let result = VestingVault::complete_unbonding(env.clone(), user.clone(), vesting_id);
    assert!(matches!(result, Err(Error::UnbondingPeriodNotElapsed)));
}

#[test]
fn test_complete_unbonding_after_period() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;
    let deposit_amount = 1000i128;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Deposit tokens
    VestingVault::deposit_to_lst_pool(
        env.clone(),
        user.clone(),
        vesting_id,
        deposit_amount,
    )
    .unwrap();

    // Request unbonding
    VestingVault::request_unbonding(env.clone(), user.clone(), vesting_id).unwrap();

    // Advance time past unbonding period
    env.ledger().set_timestamp(env.ledger().timestamp() + unbonding_period + 1);

    // Complete unbonding
    let result = VestingVault::complete_unbonding(env.clone(), user.clone(), vesting_id);
    assert!(result.is_ok());

    // Verify user shares were reset
    let user_shares = VestingVault::get_user_lst_shares(env.clone(), &user, vesting_id).unwrap();
    assert_eq!(user_shares.shares, 0);
    assert!(!user_shares.unbonding_pending);
}

#[test]
fn test_unbonding_queue_rate_limit() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Create 100 unbonding requests to fill the queue
    for i in 0..100 {
        let user = Address::generate(&env);
        VestingVault::deposit_to_lst_pool(env.clone(), user.clone(), vesting_id, 100i128).unwrap();
        VestingVault::request_unbonding(env.clone(), user, vesting_id).unwrap();
    }

    // Try to add one more - should fail due to rate limit
    let user_101 = Address::generate(&env);
    VestingVault::deposit_to_lst_pool(env.clone(), user_101.clone(), vesting_id, 100i128).unwrap();
    let result = VestingVault::request_unbonding(env.clone(), user_101, vesting_id);
    assert!(matches!(result, Err(Error::UnbondingQueueFull)));
}

#[test]
fn test_rebasing_token_simulation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // User1 deposits 1000 tokens
    VestingVault::deposit_to_lst_pool(env.clone(), user1.clone(), vesting_id, 1000i128).unwrap();

    // User2 deposits 1000 tokens
    VestingVault::deposit_to_lst_pool(env.clone(), user2.clone(), vesting_id, 1000i128).unwrap();

    // Verify pool state
    let pool_shares = VestingVault::get_lst_pool_shares(env.clone(), vesting_id).unwrap();
    assert_eq!(pool_shares.total_shares, 2000);
    assert_eq!(pool_shares.total_underlying, 2000);

    // Simulate rewards by manually updating pool state (in real scenario, this would come from staking contract)
    // This simulates a rebasing token where the underlying balance increases
    let mut updated_pool = pool_shares;
    updated_pool.total_underlying = 2200; // 10% yield
    updated_pool.last_compounded_at = env.ledger().timestamp();
    updated_pool.exchange_rate_snapshot = (2200 * 10_000_000i128) / 2000;
    updated_pool.snapshot_timestamp = env.ledger().timestamp();
    VestingVault::set_lst_pool_shares(env.clone(), vesting_id, &updated_pool);

    // Advance time
    env.ledger().set_timestamp(env.ledger().timestamp() + 7200);

    // Compound rewards
    VestingVault::compound_lst_rewards(env.clone(), vesting_id).unwrap();

    // Calculate user1's claimable amount based on shares
    let user1_claim = VestingVault::calculate_shares_based_claim(env.clone(), &user1, vesting_id).unwrap();
    // user1 has 1000 shares out of 2000 total = 50% of pool
    // Pool now has 2200 underlying, so user1 should get 1100
    assert_eq!(user1_claim, 1100);

    // Calculate user2's claimable amount
    let user2_claim = VestingVault::calculate_shares_based_claim(env.clone(), &user2, vesting_id).unwrap();
    assert_eq!(user2_claim, 1100);

    // Verify internal accounting matches actual balances
    let final_pool = VestingVault::get_lst_pool_shares(env.clone(), vesting_id).unwrap();
    assert_eq!(final_pool.total_underlying, 2200);
    assert_eq!(final_pool.total_shares, 2000);
}

#[test]
fn test_shares_based_claim_calculation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Deposit tokens
    VestingVault::deposit_to_lst_pool(env.clone(), user.clone(), vesting_id, 1000i128).unwrap();

    // Calculate claimable amount
    let claimable = VestingVault::calculate_shares_based_claim(env.clone(), &user, vesting_id).unwrap();
    assert_eq!(claimable, 1000);

    // Simulate yield by updating pool
    let mut pool = VestingVault::get_lst_pool_shares(env.clone(), vesting_id).unwrap();
    pool.total_underlying = 1100; // 10% yield
    VestingVault::set_lst_pool_shares(env.clone(), vesting_id, &pool);

    // Calculate claimable amount after yield
    let claimable_after_yield = VestingVault::calculate_shares_based_claim(env.clone(), &user, vesting_id).unwrap();
    assert_eq!(claimable_after_yield, 1100);
}

#[test]
fn test_no_user_shares_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let lst_token = Address::generate(&env);
    let base_token = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let vesting_id = 1u32;
    let unbonding_period = 604800u64;

    env.mock_all_auths();

    // Configure LST compounding
    VestingVault::configure_lst_compounding(
        env.clone(),
        admin.clone(),
        vesting_id,
        lst_token.clone(),
        base_token.clone(),
        staking_contract.clone(),
        unbonding_period,
    );

    // Try to calculate claim without depositing
    let result = VestingVault::calculate_shares_based_claim(env.clone(), &user, vesting_id);
    assert!(matches!(result, Err(Error::NoUserShares)));
}
