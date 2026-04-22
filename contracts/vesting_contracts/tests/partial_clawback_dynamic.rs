use soroban_sdk::{testutils::Address as _, vec, Address, Env};

use vesting_contracts::{BatchCreateData, ScheduleConfig, VestingContract, VestingContractClient, AdminAction, AssetAllocationEntry};

fn setup(env: &Env) -> (VestingContractClient<'static>, Address, Address) {
    env.mock_all_auths();

    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token = Address::generate(env);
    // initialize multisig (single admin but requires proposal flow)
    let mut admins = vec![env];
    admins.push_back(admin.clone());
    client.initialize_multisig(&admins, &1u32, &1_000_000i128);

    (client, admin, token)
}

#[test]
fn test_partial_clawback_dynamic_basic() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    let beneficiary = Address::generate(&env);
    let start = env.ledger().timestamp();
    let end = start + 1000; // 1000 seconds duration

    // Create a vault with 1000 tokens over 1000 seconds (1 token/second)
    let cfg = ScheduleConfig {
        owner: beneficiary.clone(),
        asset_basket: soroban_sdk::vec![&env, AssetAllocationEntry {
            asset_id: token,
            total_amount: 1000i128,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: start,
        end_time: end,
        keeper_fee: 0i128,
        is_revocable: true,
        is_transferable: false,
    };
    
    let action = AdminAction::AddBeneficiary(beneficiary.clone(), cfg);
    client.propose_admin_action(&admin, &action);
    
    // Get the vault ID (assuming it's 0 for first vault)
    let vault_id = 0u64;

    // Advance time to halfway point (500 seconds)
    env.ledger().set_timestamp(start + 500);

    // Check vested amount before clawback (should be 500 tokens)
    let claimable_before = client.calculate_claimable(&vault_id);
    assert_eq!(claimable_before, 500i128);

    // Perform partial clawback of 200 tokens from unvested portion
    let treasury = Address::generate(&env);
    client.partial_clawback_dynamic(&admin, &vault_id, &200i128, &treasury);

    // Check that clawback adjustment data is stored
    let clawback_adj = client.get_clawback_adjustment(&vault_id);
    assert!(clawback_adj.clawback_time == start + 500);
    assert_eq!(clawback_adj.clawback_amount, 200i128);
    assert_eq!(clawback_adj.original_total_amount, 1000i128);
    assert_eq!(clawback_adj.remaining_tokens, 300i128); // 500 unvested - 200 clawback

    // Advance time another 250 seconds (750 total)
    env.ledger().set_timestamp(start + 750);

    // Check vested amount after clawback with dynamic rate
    // Should be: 500 (vested before clawback) + (300 * 250 / 500) = 500 + 150 = 650
    let claimable_after = client.calculate_claimable(&vault_id);
    assert_eq!(claimable_after, 650i128);

    // Advance to end time
    env.ledger().set_timestamp(end);

    // Final claimable should be original total minus clawback: 1000 - 200 = 800
    let final_claimable = client.calculate_claimable(&vault_id);
    assert_eq!(final_claimable, 800i128);
}

#[test]
fn test_partial_clawback_dynamic_early() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    let beneficiary = Address::generate(&env);
    let start = env.ledger().timestamp();
    let end = start + 1000;

    let cfg = ScheduleConfig {
        owner: beneficiary.clone(),
        asset_basket: soroban_sdk::vec![&env, AssetAllocationEntry {
            asset_id: token,
            total_amount: 1000i128,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: start,
        end_time: end,
        keeper_fee: 0i128,
        is_revocable: true,
        is_transferable: false,
    };
    
    let action = AdminAction::AddBeneficiary(beneficiary.clone(), cfg);
    client.propose_admin_action(&admin, &action);
    
    let vault_id = 0u64;

    // Advance only 100 seconds (10% vested)
    env.ledger().set_timestamp(start + 100);

    // Clawback 300 tokens from unvested portion
    let treasury = Address::generate(&env);
    client.partial_clawback_dynamic(&admin, &vault_id, &300i128, &treasury);

    // Advance to end time
    env.ledger().set_timestamp(end);

    // Final claimable should be: 100 (vested before) + 600 (remaining) = 700
    let final_claimable = client.calculate_claimable(&vault_id);
    assert_eq!(final_claimable, 700i128);
}

#[test]
fn test_partial_clawback_dynamic_late() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    let beneficiary = Address::generate(&env);
    let start = env.ledger().timestamp();
    let end = start + 1000;

    let cfg = ScheduleConfig {
        owner: beneficiary.clone(),
        asset_basket: soroban_sdk::vec![&env, AssetAllocationEntry {
            asset_id: token,
            total_amount: 1000i128,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: start,
        end_time: end,
        keeper_fee: 0i128,
        is_revocable: true,
        is_transferable: false,
    };
    
    let action = AdminAction::AddBeneficiary(beneficiary.clone(), cfg);
    client.propose_admin_action(&admin, &action);
    
    let vault_id = 0u64;

    // Advance 800 seconds (80% vested)
    env.ledger().set_timestamp(start + 800);

    // Clawback 100 tokens from unvested portion (only 200 available)
    let treasury = Address::generate(&env);
    client.partial_clawback_dynamic(&admin, &vault_id, &100i128, &treasury);

    // Advance to end time
    env.ledger().set_timestamp(end);

    // Final claimable should be: 800 (vested before) + 100 (remaining) = 900
    let final_claimable = client.calculate_claimable(&vault_id);
    assert_eq!(final_claimable, 900i128);
}

#[test]
#[should_panic(expected = "clawback_amount exceeds available unvested tokens")]
fn test_partial_clawback_exceeds_unvested() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    let beneficiary = Address::generate(&env);
    let start = env.ledger().timestamp();
    let end = start + 1000;

    let cfg = ScheduleConfig {
        owner: beneficiary.clone(),
        asset_basket: soroban_sdk::vec![&env, AssetAllocationEntry {
            asset_id: token,
            total_amount: 1000i128,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: start,
        end_time: end,
        keeper_fee: 0i128,
        is_revocable: true,
        is_transferable: false,
    };
    
    let action = AdminAction::AddBeneficiary(beneficiary.clone(), cfg);
    client.propose_admin_action(&admin, &action);
    
    let vault_id = 0u64;

    // Advance 800 seconds (80% vested, only 200 unvested)
    env.ledger().set_timestamp(start + 800);

    // Try to clawback 300 tokens (more than available unvested)
    let treasury = Address::generate(&env);
    client.partial_clawback_dynamic(&admin, &vault_id, &300i128, &treasury);
}

#[test]
#[should_panic(expected = "Vault is irrevocable")]
fn test_partial_clawback_irrevocable() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    let beneficiary = Address::generate(&env);
    let start = env.ledger().timestamp();
    let end = start + 1000;

    let cfg = ScheduleConfig {
        owner: beneficiary.clone(),
        asset_basket: soroban_sdk::vec![&env, AssetAllocationEntry {
            asset_id: token,
            total_amount: 1000i128,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: start,
        end_time: end,
        keeper_fee: 0i128,
        is_revocable: false, // Irrevocable vault
        is_transferable: false,
    };
    
    let action = AdminAction::AddBeneficiary(beneficiary.clone(), cfg);
    client.propose_admin_action(&admin, &action);
    
    let vault_id = 0u64;

    // Try to clawback from irrevocable vault
    let treasury = Address::generate(&env);
    client.partial_clawback_dynamic(&admin, &vault_id, &100i128, &treasury);
}

#[test]
fn test_partial_clawback_mathematical_precision() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    let beneficiary = Address::generate(&env);
    let start = env.ledger().timestamp();
    let end = start + 1000;

    // Use amounts that test precision
    let cfg = ScheduleConfig {
        owner: beneficiary.clone(),
        asset_basket: soroban_sdk::vec![&env, AssetAllocationEntry {
            asset_id: token,
            total_amount: 1000000i128, // 1M tokens
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: start,
        end_time: end,
        keeper_fee: 0i128,
        is_revocable: true,
        is_transferable: false,
    };
    
    let action = AdminAction::AddBeneficiary(beneficiary.clone(), cfg);
    client.propose_admin_action(&admin, &action);
    
    let vault_id = 0u64;

    // Advance to exactly 1/3 point
    env.ledger().set_timestamp(start + 333);

    // Clawback 100000 tokens
    let treasury = Address::generate(&env);
    client.partial_clawback_dynamic(&admin, &vault_id, &100000i128, &treasury);

    // Advance to 2/3 point
    env.ledger().set_timestamp(start + 666);

    // Should have: 333000 (vested before) + (566667 * 333 / 667) = 333000 + 283000 = 616000
    let claimable = client.calculate_claimable(&vault_id);
    
    // Verify mathematical correctness (within rounding tolerance)
    assert!(claimable >= 615000i128 && claimable <= 617000i128);
}

#[test]
fn test_multiple_clawbacks() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    let beneficiary = Address::generate(&env);
    let start = env.ledger().timestamp();
    let end = start + 1000;

    let cfg = ScheduleConfig {
        owner: beneficiary.clone(),
        asset_basket: soroban_sdk::vec![&env, AssetAllocationEntry {
            asset_id: token,
            total_amount: 1000i128,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: start,
        end_time: end,
        keeper_fee: 0i128,
        is_revocable: true,
        is_transferable: false,
    };
    
    let action = AdminAction::AddBeneficiary(beneficiary.clone(), cfg);
    client.propose_admin_action(&admin, &action);
    
    let vault_id = 0u64;

    // First clawback at 25% point
    env.ledger().set_timestamp(start + 250);
    let treasury = Address::generate(&env);
    client.partial_clawback_dynamic(&admin, &vault_id, &100i128, &treasury);

    // Second clawback at 50% point
    env.ledger().set_timestamp(start + 500);
    client.partial_clawback_dynamic(&admin, &vault_id, &100i128, &treasury);

    // Advance to end
    env.ledger().set_timestamp(end);

    // Final should be: 1000 - 100 - 100 = 800
    let final_claimable = client.calculate_claimable(&vault_id);
    assert_eq!(final_claimable, 800i128);
}
