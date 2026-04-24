use soroban_sdk::{vec, Address, Env, String};
use crate::{
    DepositToYieldAdapter, LendingProtocol, YieldPosition, VaultYieldSummary,
    AdapterDataKey, AdapterError
};

#[test]
fn test_initialization() {
    let env = Env::default();
    let admin = Address::random(&env);
    let vesting_contract = Address::random(&env);
    let yield_treasury = Address::random(&env);

    DepositToYieldAdapter::initialize(env.clone(), admin.clone(), vesting_contract.clone(), yield_treasury.clone());

    // Verify storage
    let stored_admin: Address = env.storage().instance().get(&AdapterDataKey::Admin).unwrap();
    assert_eq!(stored_admin, admin);

    let stored_vesting: Address = env.storage().instance().get(&AdapterDataKey::VestingContract).unwrap();
    assert_eq!(stored_vesting, vesting_contract);

    let stored_treasury: Address = env.storage().instance().get(&AdapterDataKey::YieldTreasury).unwrap();
    assert_eq!(stored_treasury, yield_treasury);

    let is_paused: bool = env.storage().instance().get(&AdapterDataKey::IsPaused).unwrap();
    assert!(!is_paused);
}

#[test]
fn test_whitelist_protocol() {
    let env = Env::default();
    let admin = Address::random(&env);
    let vesting_contract = Address::random(&env);
    let yield_treasury = Address::random(&env);
    let protocol_address = Address::random(&env);
    let asset_address = Address::random(&env);

    DepositToYieldAdapter::initialize(env.clone(), admin.clone(), vesting_contract, yield_treasury);

    let protocol = LendingProtocol {
        address: protocol_address.clone(),
        name: String::from_str(&env, "USDC Lending Pool"),
        is_active: true,
        risk_rating: 1, // Low risk
        supported_assets: vec![&env, asset_address.clone()],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };

    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), protocol.clone());

    // Verify protocol is stored
    let stored_protocol = DepositToYieldAdapter::get_whitelisted_protocol(&env, &protocol_address);
    assert_eq!(stored_protocol.name, protocol.name);
    assert_eq!(stored_protocol.risk_rating, 1);
}

#[test]
#[should_panic(expected = "Risk rating too high")]
fn test_whitelist_high_risk_protocol() {
    let env = Env::default();
    let admin = Address::random(&env);
    let vesting_contract = Address::random(&env);
    let yield_treasury = Address::random(&env);
    let protocol_address = Address::random(&env);
    let asset_address = Address::random(&env);

    DepositToYieldAdapter::initialize(env.clone(), admin.clone(), vesting_contract, yield_treasury);

    let protocol = LendingProtocol {
        address: protocol_address,
        name: String::from_str(&env, "High Risk Pool"),
        is_active: true,
        risk_rating: 4, // High risk - should be rejected
        supported_assets: vec![&env, asset_address],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };

    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin, protocol);
}

#[test]
fn test_deposit_to_yield() {
    let env = Env::default();
    let admin = Address::random(&env);
    let vesting_contract = Address::random(&env);
    let yield_treasury = Address::random(&env);
    let protocol_address = Address::random(&env);
    let asset_address = Address::random(&env);

    DepositToYieldAdapter::initialize(env.clone(), admin.clone(), vesting_contract, yield_treasury);

    let protocol = LendingProtocol {
        address: protocol_address.clone(),
        name: String::from_str(&env, "USDC Lending Pool"),
        is_active: true,
        risk_rating: 1,
        supported_assets: vec![&env, asset_address.clone()],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };

    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), protocol);

    let vault_id = 1;
    let deposit_amount = 5000;

    let shares_received = DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        protocol_address.clone(),
        asset_address.clone(),
        deposit_amount,
    );

    assert_eq!(shares_received, deposit_amount); // 1:1 ratio in placeholder

    // Verify position is stored
    let positions = DepositToYieldAdapter::get_vault_positions(env.clone(), vault_id);
    assert_eq!(positions.len(), 1);

    let position = positions.get(0).unwrap();
    assert_eq!(position.protocol_address, protocol_address);
    assert_eq!(position.asset_address, asset_address);
    assert_eq!(position.deposited_amount, deposit_amount);
    assert_eq!(position.shares, shares_received);

    // Verify yield summary
    let summary = DepositToYieldAdapter::get_vault_yield_summary(env.clone(), vault_id);
    assert_eq!(summary.total_deposited, deposit_amount);
    assert_eq!(summary.total_yield_accumulated, 0);
    assert_eq!(summary.active_positions.len(), 1);
}

#[test]
fn test_claim_yield() {
    let env = Env::default();
    let admin = Address::random(&env);
    let vesting_contract = Address::random(&env);
    let yield_treasury = Address::random(&env);
    let protocol_address = Address::random(&env);
    let asset_address = Address::random(&env);

    DepositToYieldAdapter::initialize(env.clone(), admin.clone(), vesting_contract, yield_treasury);

    let protocol = LendingProtocol {
        address: protocol_address.clone(),
        name: String::from_str(&env, "USDC Lending Pool"),
        is_active: true,
        risk_rating: 1,
        supported_assets: vec![&env, asset_address.clone()],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };

    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), protocol);

    let vault_id = 1;
    let deposit_amount = 5000;

    // First deposit
    DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        protocol_address.clone(),
        asset_address.clone(),
        deposit_amount,
    );

    // Claim yield
    let claimed_yield = DepositToYieldAdapter::claim_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        protocol_address.clone(),
        asset_address.clone(),
    );

    // In placeholder, yield is 1% of deposited amount
    let expected_yield = deposit_amount / 100;
    assert_eq!(claimed_yield, expected_yield);

    // Verify yield summary is updated
    let summary = DepositToYieldAdapter::get_vault_yield_summary(env.clone(), vault_id);
    assert_eq!(summary.total_yield_accumulated, expected_yield);

    // Verify position is updated
    let positions = DepositToYieldAdapter::get_vault_positions(env.clone(), vault_id);
    let position = positions.get(0).unwrap();
    assert_eq!(position.accumulated_yield, expected_yield);
    assert!(position.last_yield_claim > 0);
}

#[test]
fn test_withdraw_position() {
    let env = Env::default();
    let admin = Address::random(&env);
    let vesting_contract = Address::random(&env);
    let yield_treasury = Address::random(&env);
    let protocol_address = Address::random(&env);
    let asset_address = Address::random(&env);

    DepositToYieldAdapter::initialize(env.clone(), admin.clone(), vesting_contract, yield_treasury);

    let protocol = LendingProtocol {
        address: protocol_address.clone(),
        name: String::from_str(&env, "USDC Lending Pool"),
        is_active: true,
        risk_rating: 1,
        supported_assets: vec![&env, asset_address.clone()],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };

    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), protocol);

    let vault_id = 1;
    let deposit_amount = 5000;

    // Deposit
    DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        protocol_address.clone(),
        asset_address.clone(),
        deposit_amount,
    );

    // Withdraw
    let (principal_withdrawn, yield_withdrawn) = DepositToYieldAdapter::withdraw_position(
        env.clone(),
        admin.clone(),
        vault_id,
        protocol_address.clone(),
        asset_address.clone(),
    );

    assert_eq!(principal_withdrawn, deposit_amount);
    let expected_yield = deposit_amount / 50; // 2% yield placeholder
    assert_eq!(yield_withdrawn, expected_yield);

    // Verify position is removed
    let positions = DepositToYieldAdapter::get_vault_positions(env.clone(), vault_id);
    assert_eq!(positions.len(), 0);

    // Verify yield summary is updated
    let summary = DepositToYieldAdapter::get_vault_yield_summary(env.clone(), vault_id);
    assert_eq!(summary.total_deposited, 0);
    assert_eq!(summary.total_yield_accumulated, expected_yield);
    assert_eq!(summary.active_positions.len(), 0);
}

#[test]
fn test_pause_functionality() {
    let env = Env::default();
    let admin = Address::random(&env);
    let vesting_contract = Address::random(&env);
    let yield_treasury = Address::random(&env);
    let protocol_address = Address::random(&env);
    let asset_address = Address::random(&env);

    DepositToYieldAdapter::initialize(env.clone(), admin.clone(), vesting_contract, yield_treasury);

    // Pause the contract
    DepositToYieldAdapter::set_pause(env.clone(), admin.clone(), true);

    // Verify contract is paused
    let is_paused: bool = env.storage().instance().get(&AdapterDataKey::IsPaused).unwrap();
    assert!(is_paused);

    // Try to whitelist protocol while paused - should panic
    let protocol = LendingProtocol {
        address: protocol_address,
        name: String::from_str(&env, "Test Pool"),
        is_active: true,
        risk_rating: 1,
        supported_assets: vec![&env, asset_address],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };

    let result = std::panic::catch_unwind(|| {
        DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), protocol);
    });
    assert!(result.is_err());

    // Unpause the contract
    DepositToYieldAdapter::set_pause(env.clone(), admin.clone(), false);

    // Verify contract is unpaused
    let is_paused: bool = env.storage().instance().get(&AdapterDataKey::IsPaused).unwrap();
    assert!(!is_paused);
}

#[test]
fn test_multiple_positions_same_vault() {
    let env = Env::default();
    let admin = Address::random(&env);
    let vesting_contract = Address::random(&env);
    let yield_treasury = Address::random(&env);
    let protocol1_address = Address::random(&env);
    let protocol2_address = Address::random(&env);
    let asset1_address = Address::random(&env);
    let asset2_address = Address::random(&env);

    DepositToYieldAdapter::initialize(env.clone(), admin.clone(), vesting_contract, yield_treasury);

    // Create two protocols
    let protocol1 = LendingProtocol {
        address: protocol1_address.clone(),
        name: String::from_str(&env, "USDC Pool"),
        is_active: true,
        risk_rating: 1,
        supported_assets: vec![&env, asset1_address.clone()],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };

    let protocol2 = LendingProtocol {
        address: protocol2_address.clone(),
        name: String::from_str(&env, "USDT Pool"),
        is_active: true,
        risk_rating: 1,
        supported_assets: vec![&env, asset2_address.clone()],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };

    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), protocol1);
    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), protocol2);

    let vault_id = 1;

    // Deposit to first protocol
    DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        protocol1_address.clone(),
        asset1_address.clone(),
        3000,
    );

    // Deposit to second protocol
    DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        protocol2_address.clone(),
        asset2_address.clone(),
        2000,
    );

    // Verify both positions exist
    let positions = DepositToYieldAdapter::get_vault_positions(env.clone(), vault_id);
    assert_eq!(positions.len(), 2);

    // Verify yield summary
    let summary = DepositToYieldAdapter::get_vault_yield_summary(env.clone(), vault_id);
    assert_eq!(summary.total_deposited, 5000);
    assert_eq!(summary.active_positions.len(), 2);
}

#[test]
fn test_position_accumulation() {
    let env = Env::default();
    let admin = Address::random(&env);
    let vesting_contract = Address::random(&env);
    let yield_treasury = Address::random(&env);
    let protocol_address = Address::random(&env);
    let asset_address = Address::random(&env);

    DepositToYieldAdapter::initialize(env.clone(), admin.clone(), vesting_contract, yield_treasury);

    let protocol = LendingProtocol {
        address: protocol_address.clone(),
        name: String::from_str(&env, "USDC Pool"),
        is_active: true,
        risk_rating: 1,
        supported_assets: vec![&env, asset_address.clone()],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };

    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), protocol);

    let vault_id = 1;

    // First deposit
    DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        protocol_address.clone(),
        asset_address.clone(),
        3000,
    );

    // Second deposit to same protocol/asset
    DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        protocol_address.clone(),
        asset_address.clone(),
        2000,
    );

    // Verify positions are accumulated (should be 1 position with combined amounts)
    let positions = DepositToYieldAdapter::get_vault_positions(env.clone(), vault_id);
    assert_eq!(positions.len(), 1);

    let position = positions.get(0).unwrap();
    assert_eq!(position.deposited_amount, 5000);
    assert_eq!(position.shares, 5000); // 1:1 ratio
}
