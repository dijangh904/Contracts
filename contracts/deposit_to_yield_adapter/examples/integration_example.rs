use soroban_sdk::{vec, Address, Env, String};
use deposit_to_yield_adapter::{
    DepositToYieldAdapter, LendingProtocol, YieldPosition, VaultYieldSummary
};

/// Example showing how to integrate the DepositToYieldAdapter with the vesting system
pub fn main() {
    let env = Env::default();
    
    // Initialize the adapter
    let admin = Address::generate(&env);
    let vesting_contract = Address::generate(&env);
    let yield_treasury = Address::generate(&env);
    
    DepositToYieldAdapter::initialize(
        env.clone(), 
        admin.clone(), 
        vesting_contract.clone(), 
        yield_treasury.clone()
    );
    
    // Step 1: Whitelist a low-risk USDC lending protocol
    let usdc_address = Address::generate(&env);
    let usdc_protocol_address = Address::generate(&env);
    
    let usdc_protocol = LendingProtocol {
        address: usdc_protocol_address.clone(),
        name: String::from_str(&env, "Compound USDC Pool"),
        is_active: true,
        risk_rating: 1, // Low risk
        supported_assets: vec![&env, usdc_address.clone()],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };
    
    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), usdc_protocol);
    
    // Step 2: Whitelist another low-risk USDT lending protocol
    let usdt_address = Address::generate(&env);
    let usdt_protocol_address = Address::generate(&env);
    
    let usdt_protocol = LendingProtocol {
        address: usdt_protocol_address.clone(),
        name: String::from_str(&env, "Aave USDT Pool"),
        is_active: true,
        risk_rating: 2, // Still low risk
        supported_assets: vec![&env, usdt_address.clone()],
        minimum_deposit: 500,
        maximum_deposit: 500000,
    };
    
    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), usdt_protocol);
    
    // Step 3: Deposit unvested tokens from vault #1 to USDC protocol
    let vault_id = 1;
    let usdc_deposit_amount = 50000; // 50,000 USDC
    
    let usdc_shares = DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        usdc_protocol_address.clone(),
        usdc_address.clone(),
        usdc_deposit_amount,
    );
    
    println!("Deposited {} USDC, received {} shares", usdc_deposit_amount, usdc_shares);
    
    // Step 4: Deposit unvested tokens from same vault to USDT protocol
    let usdt_deposit_amount = 30000; // 30,000 USDT
    
    let usdt_shares = DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        usdt_protocol_address.clone(),
        usdt_address.clone(),
        usdt_deposit_amount,
    );
    
    println!("Deposited {} USDT, received {} shares", usdt_deposit_amount, usdt_shares);
    
    // Step 5: Check vault positions and yield summary
    let positions = DepositToYieldAdapter::get_vault_positions(env.clone(), vault_id);
    println!("Vault {} has {} active positions:", vault_id, positions.len());
    
    for (i, position) in positions.iter().enumerate() {
        println!("  Position {}: {} tokens in {} protocol", 
                i, position.deposited_amount, position.protocol_address);
    }
    
    let summary = DepositToYieldAdapter::get_vault_yield_summary(env.clone(), vault_id);
    println!("Vault {} summary:", vault_id);
    println!("  Total deposited: {}", summary.total_deposited);
    println!("  Total yield accumulated: {}", summary.total_yield_accumulated);
    println!("  Active positions: {}", summary.active_positions.len());
    
    // Step 6: Claim yield from USDC position
    let usdc_yield = DepositToYieldAdapter::claim_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        usdc_protocol_address.clone(),
        usdc_address.clone(),
    );
    
    println!("Claimed {} USDC yield", usdc_yield);
    
    // Step 7: Claim yield from USDT position
    let usdt_yield = DepositToYieldAdapter::claim_yield(
        env.clone(),
        admin.clone(),
        vault_id,
        usdt_protocol_address.clone(),
        usdt_address.clone(),
    );
    
    println!("Claimed {} USDT yield", usdt_yield);
    
    // Step 8: Check updated summary
    let updated_summary = DepositToYieldAdapter::get_vault_yield_summary(env.clone(), vault_id);
    println!("Updated vault {} summary:", vault_id);
    println!("  Total deposited: {}", updated_summary.total_deposited);
    println!("  Total yield accumulated: {}", updated_summary.total_yield_accumulated);
    
    // Step 9: Withdraw from USDC position (emergency or maturity)
    let (usdc_principal, usdc_yield_withdrawn) = DepositToYieldAdapter::withdraw_position(
        env.clone(),
        admin.clone(),
        vault_id,
        usdc_protocol_address.clone(),
        usdc_address.clone(),
    );
    
    println!("Withdrew from USDC position:");
    println!("  Principal: {}", usdc_principal);
    println!("  Yield: {}", usdc_yield_withdrawn);
    
    // Step 10: Check final positions
    let final_positions = DepositToYieldAdapter::get_vault_positions(env.clone(), vault_id);
    println!("Final vault {} has {} active positions", vault_id, final_positions.len());
    
    let final_summary = DepositToYieldAdapter::get_vault_yield_summary(env.clone(), vault_id);
    println!("Final vault {} summary:", vault_id);
    println!("  Total deposited: {}", final_summary.total_deposited);
    println!("  Total yield accumulated: {}", final_summary.total_yield_accumulated);
}

/// Example showing risk management and admin controls
pub fn risk_management_example() {
    let env = Env::default();
    
    let admin = Address::generate(&env);
    let vesting_contract = Address::generate(&env);
    let yield_treasury = Address::generate(&env);
    
    DepositToYieldAdapter::initialize(
        env.clone(), 
        admin.clone(), 
        vesting_contract, 
        yield_treasury
    );
    
    // Try to whitelist a high-risk protocol (should fail)
    let high_risk_protocol_address = Address::generate(&env);
    let asset_address = Address::generate(&env);
    
    let high_risk_protocol = LendingProtocol {
        address: high_risk_protocol_address,
        name: String::from_str(&env, "High Yield DeFi Pool"),
        is_active: true,
        risk_rating: 4, // High risk - should be rejected
        supported_assets: vec![&env, asset_address],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };
    
    // This would panic due to high risk rating
    // DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), high_risk_protocol);
    
    // Pause the contract for emergency
    DepositToYieldAdapter::set_pause(env.clone(), admin.clone(), true);
    
    // Try to perform operations while paused (should fail)
    // DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), low_risk_protocol);
    
    // Unpause the contract
    DepositToYieldAdapter::set_pause(env.clone(), admin.clone(), false);
    
    println!("Risk management example completed");
}

/// Example showing multiple vaults with different strategies
pub fn multi_vault_example() {
    let env = Env::default();
    
    let admin = Address::generate(&env);
    let vesting_contract = Address::generate(&env);
    let yield_treasury = Address::generate(&env);
    
    DepositToYieldAdapter::initialize(
        env.clone(), 
        admin.clone(), 
        vesting_contract, 
        yield_treasury
    );
    
    // Setup multiple protocols
    let usdc_protocol_address = Address::generate(&env);
    let usdc_address = Address::generate(&env);
    
    let usdc_protocol = LendingProtocol {
        address: usdc_protocol_address.clone(),
        name: String::from_str(&env, "Compound USDC Pool"),
        is_active: true,
        risk_rating: 1,
        supported_assets: vec![&env, usdc_address.clone()],
        minimum_deposit: 1000,
        maximum_deposit: 1000000,
    };
    
    DepositToYieldAdapter::whitelist_protocol(env.clone(), admin.clone(), usdc_protocol);
    
    // Vault #1: Conservative strategy - USDC only
    let vault1_id = 1;
    let vault1_deposit = 100000; // 100k USDC
    
    DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault1_id,
        usdc_protocol_address.clone(),
        usdc_address.clone(),
        vault1_deposit,
    );
    
    // Vault #2: Smaller allocation
    let vault2_id = 2;
    let vault2_deposit = 25000; // 25k USDC
    
    DepositToYieldAdapter::deposit_to_yield(
        env.clone(),
        admin.clone(),
        vault2_id,
        usdc_protocol_address.clone(),
        usdc_address.clone(),
        vault2_deposit,
    );
    
    // Check each vault's summary
    for vault_id in [vault1_id, vault2_id] {
        let summary = DepositToYieldAdapter::get_vault_yield_summary(env.clone(), vault_id);
        println!("Vault {} summary:", vault_id);
        println!("  Total deposited: {}", summary.total_deposited);
        println!("  Active positions: {}", summary.active_positions.len());
    }
    
    println!("Multi-vault example completed");
}
