#[cfg(test)]
mod performance_cliff_tests {
    use super::*;
    use crate::{
        ComparisonOperator, Milestone, OracleClient, PerformanceCliff, VestingContract,
        VestingContractClient,
    };
    use soroban_sdk::{
        testutils::Address as _,
        testutils::Ledger as _,
        Address,
        Env,
        Symbol,
        vec,
    };

    fn setup_test(env: &Env) -> (Address, Address, VestingContractClient<'static>) {
        env.mock_all_auths();
        let admin = Address::generate(env);
        let beneficiary = Address::generate(env);

        let contract_id = env.register(VestingContract, ());
        let client = VestingContractClient::new(env, &contract_id);

        client.initialize(&admin, &1000000);

        let token_admin = Address::generate(env);
        let token_addr = env.register_stellar_asset_contract_v2(token_admin).address();
        client.set_token(&token_addr);
        
        let token_client = soroban_sdk::token::StellarAssetClient::new(env, &token_addr);
        token_client.mint(&admin, &1000000);

        (admin, beneficiary, client)
    }

    #[test]
    fn test_performance_cliff_creation() {
        let env = Env::default();
        let (admin, beneficiary, client) = setup_test(&env);

        // Create a performance cliff with TVL condition
        let oracle_address = admin.clone();
        let tvl_condition = OracleClient::create_tvl_condition(
            oracle_address.clone(),
            1000000,
            ComparisonOperator::GreaterThanOrEqual
        );

        let conditions = vec![&env, tvl_condition];
        let cliff = PerformanceCliff {
            conditions: conditions.clone(),
            require_all: true,
            fallback_time: 1640995200,
        };

        // Create vault with performance cliff
        let vault_id = client.create_vault_with_cliff(
            &beneficiary,
            &100000,
            &1640995200,
            &1672531200,
            &1000,
            &true,
            &false,
            &0,
            &cliff
        );

        // Verify cliff was set
        let retrieved_cliff = client.get_performance_cliff(&vault_id);
        assert!(retrieved_cliff.is_some());

        // Check cliff status (should be false since oracle returns 0)
        let cliff_passed = client.is_cliff_passed(&vault_id);
        assert!(!cliff_passed);

        // Verify no tokens are claimable before cliff is passed
        let claimable = client.get_claimable_amount(&vault_id);
        assert_eq!(claimable, 0);
    }

    #[test]
    fn test_multiple_oracle_conditions() {
        let env = Env::default();
        let (admin, beneficiary, client) = setup_test(&env);

        // Create multiple conditions
        let tvl_oracle = admin.clone();
        let price_oracle = admin.clone();

        let tvl_condition = OracleClient::create_tvl_condition(
            tvl_oracle,
            1000000,
            ComparisonOperator::GreaterThanOrEqual
        );

        let price_condition = OracleClient::create_price_condition(
            price_oracle,
            100,
            ComparisonOperator::GreaterThan,
            Some(Symbol::new(&env, "TOKEN"))
        );

        let conditions = vec![&env, tvl_condition, price_condition];

        // Test AND logic (require all)
        let and_cliff = PerformanceCliff {
            conditions: conditions.clone(),
            require_all: true,
            fallback_time: 1640995200,
        };

        // Test OR logic (require any)
        let or_cliff = PerformanceCliff {
            conditions: conditions.clone(),
            require_all: false,
            fallback_time: 1640995200,
        };

        let vault_id_and = client.create_vault_with_cliff(
            &beneficiary,
            &100000,
            &1640995200,
            &1672531200,
            &1000,
            &true,
            &false,
            &0,
            &and_cliff
        );

        let vault_id_or = client.create_vault_with_cliff(
            &beneficiary,
            &100000,
            &1640995200,
            &1672531200,
            &1000,
            &true,
            &false,
            &0,
            &or_cliff
        );

        // Both should fail since oracle returns 0
        assert!(!client.is_cliff_passed(&vault_id_and));
        assert!(!client.is_cliff_passed(&vault_id_or));
    }

    #[test]
    fn test_fallback_time_behavior() {
        let env = Env::default();
        let (admin, beneficiary, client) = setup_test(&env);

        // Create cliff with past fallback time
        let oracle_address = admin.clone();
        let condition = OracleClient::create_tvl_condition(
            oracle_address,
            1000000,
            ComparisonOperator::GreaterThanOrEqual
        );

        let conditions = vec![&env, condition];
        let cliff = PerformanceCliff {
            conditions: conditions.clone(),
            require_all: true,
            fallback_time: 1000000,
        };

        let vault_id = client.create_vault_with_cliff(
            &beneficiary,
            &100000,
            &1640995200,
            &1672531200,
            &1000,
            &true,
            &false,
            &0,
            &cliff
        );

        // Advance ledger time past fallback time
        env.ledger().with_mut(|li| li.timestamp = 1000001);

        // Cliff should pass due to fallback time
        let cliff_passed = client.is_cliff_passed(&vault_id);
        assert!(cliff_passed);

        // Tokens should be claimable (linear vesting from start_time)
        env.ledger().with_mut(|li| li.timestamp = 1640995200 + 86400);
        let claimable = client.get_claimable_amount(&vault_id);
        assert!(claimable > 0);
    }

    #[test]
    fn test_milestone_with_performance_cliff() {
        let env = Env::default();
        let (admin, beneficiary, client) = setup_test(&env);

        // Create performance cliff
        let oracle_address = admin.clone();
        let condition = OracleClient::create_tvl_condition(
            oracle_address,
            1000000,
            ComparisonOperator::GreaterThanOrEqual
        );

        let conditions = vec![&env, condition];
        let cliff = PerformanceCliff {
            conditions: conditions.clone(),
            require_all: true,
            fallback_time: 1640995200,
        };

        let vault_id = client.create_vault_with_cliff(
            &beneficiary,
            &100000,
            &1640995200,
            &1672531200,
            &1000,
            &true,
            &false,
            &0,
            &cliff
        );

        // Set milestones
        let milestone1 = Milestone {
            id: 1,
            percentage: 25,
            is_unlocked: false,
        };
        let milestone2 = Milestone {
            id: 2,
            percentage: 50,
            is_unlocked: false,
        };

        let milestones = vec![&env, milestone1, milestone2];
        client.set_milestones(&vault_id, &milestones);

        // Even with milestones, no tokens should be claimable before cliff
        let claimable = client.get_claimable_amount(&vault_id);
        assert_eq!(claimable, 0);

        // Unlock first milestone after cliff passes
        client.unlock_milestone(&vault_id, &1);

        // Still no tokens claimable since cliff not passed
        let claimable = client.get_claimable_amount(&vault_id);
        assert_eq!(claimable, 0);
    }
}
