#[cfg(test)]
mod tests {
    use soroban_sdk::Env;
    use soroban_sdk::Address;
    use crate::VestingVault;
    use crate::types::{LeaverType, ScheduleTerminated};
    use crate::storage::is_schedule_terminated;

    #[test]
    fn test_good_leaver_termination() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VestingVault);
        let client = VestingVaultClient::new(&env, &contract_id);

        // Setup test addresses
        let admin = Address::random(&env);
        let beneficiary = Address::random(&env);
        let treasury = Address::random(&env);
        let vesting_id = 1u32;

        // Mock admin authentication (in real test, this would be handled differently)
        env.mock_auths(&[
            (&admin, &contract_id, &"terminate_schedule".into()),
        ]);

        // Test Good Leaver termination
        client.terminate_schedule(
            &admin,
            &beneficiary,
            &vesting_id,
            &LeaverType::GoodLeaver,
            &treasury,
        );

        // Verify schedule is marked as terminated
        assert!(is_schedule_terminated(&env, vesting_id));

        // Verify termination event was emitted
        let events = env.events().all();
        assert_eq!(events.len(), 1);
        
        let event = events.get(0).unwrap();
        assert_eq!(event.topics.get(0), Some(&"ScheduleTerminated".into()));
        
        // Parse the event data (this would need proper event parsing in real implementation)
        // For now, just verify the event exists
    }

    #[test]
    fn test_bad_leaver_termination() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VestingVault);
        let client = VestingVaultClient::new(&env, &contract_id);

        // Setup test addresses
        let admin = Address::random(&env);
        let beneficiary = Address::random(&env);
        let treasury = Address::random(&env);
        let vesting_id = 2u32;

        // Mock admin authentication
        env.mock_auths(&[
            (&admin, &contract_id, &"terminate_schedule".into()),
        ]);

        // Test Bad Leaver termination
        client.terminate_schedule(
            &admin,
            &beneficiary,
            &vesting_id,
            &LeaverType::BadLeaver,
            &treasury,
        );

        // Verify schedule is marked as terminated
        assert!(is_schedule_terminated(&env, vesting_id));

        // Verify termination event was emitted
        let events = env.events().all();
        assert_eq!(events.len(), 1);
        
        let event = events.get(0).unwrap();
        assert_eq!(event.topics.get(0), Some(&"ScheduleTerminated".into()));
    }

    #[test]
    fn test_duplicate_termination_fails() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VestingVault);
        let client = VestingVaultClient::new(&env, &contract_id);

        // Setup test addresses
        let admin = Address::random(&env);
        let beneficiary = Address::random(&env);
        let treasury = Address::random(&env);
        let vesting_id = 3u32;

        // Mock admin authentication
        env.mock_auths(&[
            (&admin, &contract_id, &"terminate_schedule".into()),
        ]);

        // First termination should succeed
        client.terminate_schedule(
            &admin,
            &beneficiary,
            &vesting_id,
            &LeaverType::GoodLeaver,
            &treasury,
        );

        // Second termination should fail
        let result = std::panic::catch_unwind(|| {
            client.terminate_schedule(
                &admin,
                &beneficiary,
                &vesting_id,
                &LeaverType::GoodLeaver,
                &treasury,
            );
        });

        assert!(result.is_err()); // Should panic with "Vesting schedule already terminated"
    }

    #[test]
    fn test_unauthorized_termination_fails() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VestingVault);
        let client = VestingVaultClient::new(&env, &contract_id);

        // Setup test addresses
        let unauthorized_user = Address::random(&env);
        let beneficiary = Address::random(&env);
        let treasury = Address::random(&env);
        let vesting_id = 4u32;

        // Don't mock authentication for unauthorized user

        // Attempt termination without proper authorization should fail
        let result = std::panic::catch_unwind(|| {
            client.terminate_schedule(
                &unauthorized_user,
                &beneficiary,
                &vesting_id,
                &LeaverType::GoodLeaver,
                &treasury,
            );
        });

        assert!(result.is_err()); // Should fail due to lack of authorization
    }

    #[test]
    fn test_termination_during_emergency_pause_fails() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VestingVault);
        let client = VestingVaultClient::new(&env, &contract_id);

        // Setup test addresses
        let admin = Address::random(&env);
        let beneficiary = Address::random(&env);
        let treasury = Address::random(&env);
        let vesting_id = 5u32;

        // Set up emergency pause (this would need proper implementation)
        // For now, we'll assume the emergency pause check works
        // In a real test, you'd set up the emergency pause state

        // Mock admin authentication
        env.mock_auths(&[
            (&admin, &contract_id, &"terminate_schedule".into()),
        ]);

        // This test would verify that termination fails during emergency pause
        // Implementation depends on how emergency pause is actually set up
        // For now, this is a placeholder for the test structure
    }

    #[test]
    fn test_is_schedule_terminated_public() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VestingVault);
        let client = VestingVaultClient::new(&env, &contract_id);

        // Setup test addresses
        let admin = Address::random(&env);
        let beneficiary = Address::random(&env);
        let treasury = Address::random(&env);
        let vesting_id = 6u32;

        // Initially should not be terminated
        assert!(!client.is_schedule_terminated_public(&vesting_id));

        // Mock admin authentication
        env.mock_auths(&[
            (&admin, &contract_id, &"terminate_schedule".into()),
        ]);

        // Terminate the schedule
        client.terminate_schedule(
            &admin,
            &beneficiary,
            &vesting_id,
            &LeaverType::GoodLeaver,
            &treasury,
        );

        // Now should be terminated
        assert!(client.is_schedule_terminated_public(&vesting_id));
    }

    #[test]
    fn test_good_leaver_vs_bad_leaver_differences() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VestingVault);
        let client = VestingVaultClient::new(&env, &contract_id);

        // Setup test addresses for Good Leaver
        let admin = Address::random(&env);
        let good_beneficiary = Address::random(&env);
        let bad_beneficiary = Address::random(&env);
        let treasury = Address::random(&env);
        let good_vesting_id = 7u32;
        let bad_vesting_id = 8u32;

        // Mock admin authentication
        env.mock_auths(&[
            (&admin, &contract_id, &"terminate_schedule".into()),
        ]);

        // Test Good Leaver
        client.terminate_schedule(
            &admin,
            &good_beneficiary,
            &good_vesting_id,
            &LeaverType::GoodLeaver,
            &treasury,
        );

        // Test Bad Leaver
        client.terminate_schedule(
            &admin,
            &bad_beneficiary,
            &bad_vesting_id,
            &LeaverType::BadLeaver,
            &treasury,
        );

        // Both should be terminated
        assert!(client.is_schedule_terminated_public(&good_vesting_id));
        assert!(client.is_schedule_terminated_public(&bad_vesting_id));

        // Check that different events were emitted
        let events = env.events().all();
        assert_eq!(events.len(), 2);

        // In a real implementation, you'd parse the events to verify:
        // 1. Good Leaver event shows unvested tokens slashed but vested tokens retained
        // 2. Bad Leaver event shows both unvested and unclaimed tokens slashed
    }
}
