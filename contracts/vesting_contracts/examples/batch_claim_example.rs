//! Batch Claim Example
//! 
//! This example demonstrates how to use the batch_claim function to optimize gas costs
//! when claiming tokens from multiple vesting schedules.

use soroban_sdk::{contract, contractimpl, Address, Env, Vec};
use vesting_contracts::{VestingContract, VestingContractClient};

#[contract]
pub struct BatchClaimExample;

#[contractimpl]
impl BatchClaimExample {
    /// Example showing batch claim for an advisor with multiple vesting schedules
    pub fn example_batch_claim(env: Env, advisor: Address) {
        let contract_address = env.current_contract_address();
        let vesting_client = VestingContractClient::new(&env, &contract_address);
        
        // Before batch_claim: Advisor would need to call claim_tokens_diversified() 
        // for each vault individually (3 separate transactions):
        // - Seed round vesting
        // - Private round vesting  
        // - Advisory round vesting
        
        // After batch_claim: Single transaction claims from all schedules
        let claimed_assets = vesting_client.batch_claim(&advisor);
        
        // claimed_assets contains aggregated amounts by token type
        // Example: [(token_address, total_claimed_amount)]
        // Instead of: [(token_address, seed_amount), (token_address, private_amount), (token_address, advisory_amount)]
        
        env.log().print(&format!("Batch claim completed for advisor"));
        env.log().print(&format!("Total asset types claimed: {}", claimed_assets.len()));
        
        for i in 0..claimed_assets.len() {
            let (token_address, amount) = claimed_assets.get(i).unwrap();
            env.log().print(&format!("Token: {:?}, Amount: {}", token_address, amount));
        }
    }
    
    /// Example showing gas optimization comparison
    pub fn gas_optimization_comparison(env: Env, advisor: Address) {
        let contract_address = env.current_contract_address();
        let vesting_client = VestingContractClient::new(&env, &contract_address);
        
        // Get all vaults for the advisor
        let vault_ids = vesting_client.get_user_vaults(&advisor);
        
        // OLD WAY: Individual claims (multiple transactions)
        // let mut total_gas_used = 0;
        // for vault_id in vault_ids.iter() {
        //     // Each claim_tokens_diversified() call costs gas
        //     vesting_client.claim_tokens_diversified(vault_id);
        //     total_gas_used += gas_per_claim; // ~50,000 gas per claim
        // }
        // Total: 3 * 50,000 = 150,000 gas for 3 vaults
        
        // NEW WAY: Batch claim (single transaction)
        let claimed_assets = vesting_client.batch_claim(&advisor);
        // Total: ~60,000 gas for all vaults combined
        // Gas savings: ~90,000 gas (60% reduction)
        
        env.log().print(&format!("Gas optimized batch claim completed"));
        env.log().print(&format!("Claimed {} asset types in single transaction", claimed_assets.len()));
    }
    
    /// Example showing error handling for edge cases
    pub fn batch_claim_edge_cases(env: Env, user: Address) {
        let contract_address = env.current_contract_address();
        let vesting_client = VestingContractClient::new(&env, &contract_address);
        
        // Case 1: User with no vaults
        let user_with_no_vaults = Address::generate(&env);
        let claimed_assets = vesting_client.batch_claim(&user_with_no_vaults);
        assert_eq!(claimed_assets.len(), 0); // Returns empty vector
        
        // Case 2: User with frozen/uninitialized vaults
        // Batch claim automatically skips these vaults and only claims from valid ones
        let claimed_assets = vesting_client.batch_claim(&user);
        
        // Case 3: User with no claimable tokens (all vested already)
        // Batch claim returns empty vector - no gas wasted on unnecessary transfers
        
        env.log().print(&format!("Edge cases handled gracefully"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, token};

    #[test]
    fn test_batch_claim_example() {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let advisor = Address::generate(&env);
        
        // Setup vesting contract
        let contract_id = env.register(VestingContract, ());
        let vesting_client = VestingContractClient::new(&env, &contract_id);
        
        // Initialize contract
        vesting_client.initialize(&admin, &1_000_000_000i128);
        
        // Create multiple vaults for advisor
        let now = env.ledger().timestamp();
        
        // Seed round vault
        vesting_client.create_vault_full(
            &advisor,
            &1000i128,
            &now,
            &(now + 1000),
            &0i128,
            &false,
            &true,
            &0u64,
        );
        
        // Private round vault  
        vesting_client.create_vault_full(
            &advisor,
            &2000i128,
            &now,
            &(now + 1000),
            &0i128,
            &false,
            &true,
            &0u64,
        );
        
        // Advisory round vault
        vesting_client.create_vault_full(
            &advisor,
            &1500i128,
            &now,
            &(now + 1000),
            &0i128,
            &false,
            &true,
            &0u64,
        );
        
        // Fast forward time to vest tokens
        env.ledger().set_timestamp(now + 1001);
        
        // Test batch claim
        let claimed_assets = vesting_client.batch_claim(&advisor);
        
        // Verify results
        assert_eq!(claimed_assets.len(), 1); // Single token type
        let (_, total_amount) = claimed_assets.get(0).unwrap();
        assert_eq!(*total_amount, 4500); // 1000 + 2000 + 1500
    }
}
