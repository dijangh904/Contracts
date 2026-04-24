#![cfg(test)]

use soroban_sdk::{symbol_short, Address, Env, Vec, String, Symbol, IntoVal, BytesN};
use vesting_contracts::{VestingContract, VestingContractClient, Error, PathPaymentConfig, PathPaymentSimulation, PathPaymentClaimEvent};

#[test]
fn test_configure_path_payment() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let intermediate_asset = Address::generate(&env);
    
    // Initialize contract
    client.initialize(&admin, &1000000i128);
    
    let mut path = Vec::new(&env);
    path.push_back(intermediate_asset);
    
    let min_destination_amount = 1000i128;
    
    // Configure path payment
    client.configure_path_payment(
        &admin,
        &usdc_asset,
        &min_destination_amount,
        &path
    );
    
    // Verify configuration
    let config = client.get_path_payment_config();
    assert!(config.is_some());
    
    let config = config.unwrap();
    assert_eq!(config.destination_asset, usdc_asset);
    assert_eq!(config.min_destination_amount, min_destination_amount);
    assert_eq!(config.path, path);
    assert!(config.enabled);
}

#[test]
fn test_disable_path_payment() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Initialize contract
    client.initialize(&admin, &1000000i128);
    
    // Configure path payment first
    client.configure_path_payment(&admin, &usdc_asset, &1000i128, &path);
    
    // Disable it
    client.disable_path_payment(&admin);
    
    // Verify it's disabled
    let config = client.get_path_payment_config();
    assert!(config.is_some());
    assert!(!config.unwrap().enabled);
}

#[test]
fn test_simulate_claim_and_swap_not_configured() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    
    // Initialize contract
    client.initialize(&admin, &1000000i128);
    
    // Try to simulate without configuration
    let simulation = client.simulate_claim_and_swap(&1u64, &Some(950i128));
    
    assert!(!simulation.can_execute);
    assert_eq!(simulation.source_amount, 0);
    assert_eq!(simulation.estimated_destination_amount, 0);
    assert!(simulation.reason.contains("Path payment not configured"));
}

#[test]
fn test_simulate_claim_and_swap_disabled() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Initialize contract
    client.initialize(&admin, &1000000i128);
    
    // Configure and then disable path payment
    client.configure_path_payment(&admin, &usdc_asset, &950i128, &path);
    client.disable_path_payment(&admin);
    
    // Try to simulate while disabled
    let simulation = client.simulate_claim_and_swap(&1u64, &Some(950i128));
    
    assert!(!simulation.can_execute);
    assert_eq!(simulation.source_amount, 0);
    assert_eq!(simulation.estimated_destination_amount, 0);
    assert!(simulation.reason.contains("Path payment disabled"));
}

#[test]
fn test_claim_and_swap_not_configured() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    // Initialize contract
    client.initialize(&admin, &1000000i128);
    
    // Try to claim without configuration
    let result = env.try_invoke_contract::<PathPaymentClaimEvent, Error>(
        &contract_id,
        &Symbol::new(&env, "claim_and_swap"),
        (1u64, Some(950i128)).into_val(&env),
    );
    
    assert!(result.is_err());
}

#[test]
fn test_claim_and_swap_insufficient_minimum() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Initialize contract
    client.initialize(&admin, &1000000i128);
    
    // Configure path payment
    client.configure_path_payment(&admin, &usdc_asset, &950i128, &path);
    
    // Try to claim with zero minimum amount
    let result = env.try_invoke_contract::<PathPaymentClaimEvent, Error>(
        &contract_id,
        &Symbol::new(&env, "claim_and_swap"),
        (1u64, Some(0i128)).into_val(&env),
    );
    
    assert!(result.is_err());
}

#[test]
fn test_path_payment_claim_history() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Initialize contract
    client.initialize(&admin, &1000000i128);
    
    // Initially should be empty
    let history = client.get_path_payment_claim_history();
    assert_eq!(history.len(), 0);
    
    // Configure path payment
    client.configure_path_payment(&admin, &usdc_asset, &950i128, &path);
    
    // History should still be empty (no claims yet)
    let history = client.get_path_payment_claim_history();
    assert_eq!(history.len(), 0);
}

#[test]
fn test_path_payment_simulation_zero_minimum() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let usdc_asset = Address::generate(&env);
    let path = Vec::new(&env);
    
    // Initialize contract
    client.initialize(&admin, &1000000i128);
    
    // Configure path payment
    client.configure_path_payment(&admin, &usdc_asset, &950i128, &path);
    
    // Try to simulate with zero minimum amount
    let simulation = client.simulate_claim_and_swap(&1u64, &Some(0i128));
    
    assert!(!simulation.can_execute);
    assert!(simulation.reason.contains("Invalid minimum amount"));
}
