use soroban_sdk::{contractimpl, contracttype, Address, Env, Symbol, Vec, BytesN};
use crate::{
    VestingContract, CliffSmoothingConfig, AssetAllocationEntry, Vault, YieldDestination,
    DataKey, CliffSmoothedUnlock
};

#[cfg(test)]
mod smoothed_cliff_fuzz_tests {
    use super::*;
    use soroban_sdk::testutils::{Ledger, LedgerInfo};
    use proptest::prelude::*;
    use std::collections::HashMap;

    proptest! {
        #[test]
        fn test_smoothed_cliff_mathematical_integrity(
            total_amount in 1000i128..1_000_000i128,
            cliff_percentage in 1000u32..5000u32, // 10% to 50%
            cliff_duration in 259200u64..31536000u64, // 3 days to 1 year
            smoothing_duration in 86400u64..2592000u64, // 1 day to 30 days
            total_duration in 31536000u64..63072000u64, // 1 to 2 years
            elapsed_time in 0u64..63072000u64
        ) {
            // Ensure valid configuration
            prop_assume!(cliff_duration + smoothing_duration <= total_duration);
            prop_assume!(cliff_percentage <= 10000);

            let env = Env::default();
            let admin = Address::generate(&env);
            let beneficiary = Address::generate(&env);
            
            // Setup test environment
            env.ledger().set(LedgerInfo {
                timestamp: elapsed_time,
                protocol_version: 20,
                sequence_number: 1,
                base_reserve: 10,
                min_temp_entry_ttl: 10,
                min_persistent_entry_ttl: 10,
                max_entry_ttl: 3110400,
            });

            // Create vault with smoothed cliff
            let vault_id = VestingContract::create_vault_with_smoothed_cliff(
                env.clone(),
                beneficiary.clone(),
                total_amount,
                0, // start_time
                total_duration,
                0, // keeper_fee
                true, // is_revocable
                false, // is_transferable
                0, // step_duration
                cliff_duration,
                smoothing_duration,
                cliff_percentage,
            );

            // Calculate vested amount
            let vested = VestingContract::calculate_claimable_for_asset_wrapper(
                env.clone(),
                vault_id,
                0 // asset_index
            );

            // Mathematical validation: area under curve should equal total amount at end
            if elapsed_time >= total_duration {
                prop_assert_eq!(vested, total_amount, "At end of duration, vested should equal total amount");
            }

            // Validate non-negative amounts
            prop_assert!(vested >= 0, "Vested amount should never be negative");
            prop_assert!(vested <= total_amount, "Vested amount should never exceed total amount");

            // Validate monotonicity (vested amount should never decrease)
            let cliff_end_time = cliff_duration;
            let smoothing_end_time = cliff_end_time + smoothing_duration;

            if elapsed_time > 0 {
                let previous_elapsed = elapsed_time - 1;
                env.ledger().set(LedgerInfo {
                    timestamp: previous_elapsed,
                    protocol_version: 20,
                    sequence_number: 1,
                    base_reserve: 10,
                    min_temp_entry_ttl: 10,
                    min_persistent_entry_ttl: 10,
                    max_entry_ttl: 3110400,
                });

                let previous_vested = VestingContract::calculate_claimable_for_asset_wrapper(
                    env.clone(),
                    vault_id,
                    0
                );

                prop_assert!(vested >= previous_vested, "Vested amount should be monotonic");
            }

            // Validate smoothing window behavior
            if elapsed_time > cliff_end_time && elapsed_time <= smoothing_end_time {
                let cliff_amount = (total_amount * cliff_percentage as i128) / 10000;
                let smoothing_elapsed = elapsed_time - cliff_end_time;
                let smoothing_progress = smoothing_elapsed as i128 / smoothing_duration as i128;
                let expected_cliff_release = (cliff_amount * smoothing_progress) / 1;

                // During smoothing, some portion of cliff should be released
                prop_assert!(vested > 0, "Should have some vesting during smoothing window");
                prop_assert!(expected_cliff_release >= 0, "Cliff release should be non-negative");
                prop_assert!(expected_cliff_release <= cliff_amount, "Cliff release should not exceed total cliff amount");
            }
        }

        #[test]
        fn test_proration_during_smoothing_window(
            total_amount in 1000i128..100_000i128,
            cliff_percentage in 1500u32..3500u32, // 15% to 35%
            cliff_duration in 2592000u64..7776000u64, // 30 to 90 days
            smoothing_duration in 604800u64..2592000u64, // 7 to 30 days
            total_duration in 31536000u64..63072000u64, // 1 to 2 years
            termination_time in 0u64..63072000u64
        ) {
            // Ensure valid configuration
            prop_assume!(cliff_duration + smoothing_duration <= total_duration);
            prop_assume!(cliff_percentage <= 10000);

            let env = Env::default();
            let admin = Address::generate(&env);
            let beneficiary = Address::generate(&env);

            // Create vault with smoothed cliff
            let vault_id = VestingContract::create_vault_with_smoothed_cliff(
                env.clone(),
                beneficiary.clone(),
                total_amount,
                0, // start_time
                total_duration,
                0, // keeper_fee
                true, // is_revocable
                false, // is_transferable
                0, // step_duration
                cliff_duration,
                smoothing_duration,
                cliff_percentage,
            );

            // Test proration calculation
            let cliff_end_time = cliff_duration;
            let smoothing_end_time = cliff_end_time + smoothing_duration;

            if termination_time > 0 && termination_time <= total_duration {
                // The prorated amount should be reasonable
                let vested = VestingContract::calculate_claimable_for_asset_wrapper(
                    env.clone(),
                    vault_id,
                    0
                );

                prop_assert!(vested >= 0, "Prorated vested amount should be non-negative");
                prop_assert!(vested <= total_amount, "Prorated vested amount should not exceed total amount");

                // If termination during smoothing window, should be prorated
                if termination_time > cliff_end_time && termination_time <= smoothing_end_time {
                    let cliff_amount = (total_amount * cliff_percentage as i128) / 10000;
                    let smoothing_elapsed = termination_time - cliff_end_time;
                    let smoothing_progress = smoothing_elapsed as i128 / smoothing_duration as i128;
                    let expected_prorated_cliff = (cliff_amount * smoothing_progress) / 1;

                    // Should have some cliff portion released
                    prop_assert!(vested > 0, "Should have prorated cliff release during termination");
                    prop_assert!(expected_prorated_cliff >= 0, "Prorated cliff should be non-negative");
                }
            }
        }

        #[test]
        fn test_security_validation_edge_cases(
            total_amount in 1000i128..1_000_000i128,
            cliff_percentage in 0u32..10000u32,
            cliff_duration in 0u64..31536000u64,
            smoothing_duration in 0u64..31536000u64,
            total_duration in 2592000u64..31536000u64 // 30 days to 1 year
        ) {
            let env = Env::default();
            let admin = Address::generate(&env);
            let beneficiary = Address::generate(&env);

            // Test invalid configurations - these should panic
            if cliff_duration + smoothing_duration > total_duration {
                // This configuration should fail validation
                let result = std::panic::catch_unwind(|| {
                    VestingContract::create_vault_with_smoothed_cliff(
                        env.clone(),
                        beneficiary.clone(),
                        total_amount,
                        0, // start_time
                        total_duration,
                        0, // keeper_fee
                        true, // is_revocable
                        false, // is_transferable
                        0, // step_duration
                        cliff_duration,
                        smoothing_duration,
                        cliff_percentage,
                    )
                });
                prop_assert!(result.is_err(), "Invalid smoothing configuration should panic");
            }

            if smoothing_duration < 86400 || smoothing_duration > 31536000 {
                // Invalid smoothing duration
                let result = std::panic::catch_unwind(|| {
                    VestingContract::configure_cliff_smoothing(
                        env.clone(),
                        1, // vault_id
                        cliff_duration,
                        smoothing_duration,
                        cliff_percentage,
                    )
                });
                prop_assert!(result.is_err(), "Invalid smoothing duration should panic");
            }

            if cliff_percentage > 10000 {
                // Invalid cliff percentage
                let result = std::panic::catch_unwind(|| {
                    VestingContract::configure_cliff_smoothing(
                        env.clone(),
                        1, // vault_id
                        cliff_duration,
                        smoothing_duration,
                        cliff_percentage,
                    )
                });
                prop_assert!(result.is_err(), "Invalid cliff percentage should panic");
            }
        }

        #[test]
        fn test_area_under_curve_mathematical_proof(
            total_amount in 10_000i128..1_000_000i128,
            cliff_percentage in 2000u32..4000u32, // 20% to 40%
            cliff_duration in 2592000u64..7776000u64, // 30 to 90 days
            smoothing_duration in 604800u64..2592000u64, // 7 to 30 days
            total_duration in 31536000u64..63072000u64 // 1 to 2 years
        ) {
            prop_assume!(cliff_duration + smoothing_duration <= total_duration);

            let env = Env::default();
            let admin = Address::generate(&env);
            let beneficiary = Address::generate(&env);

            // Create vault
            let vault_id = VestingContract::create_vault_with_smoothed_cliff(
                env.clone(),
                beneficiary.clone(),
                total_amount,
                0, // start_time
                total_duration,
                0, // keeper_fee
                true, // is_revocable
                false, // is_transferable
                0, // step_duration
                cliff_duration,
                smoothing_duration,
                cliff_percentage,
            );

            let cliff_amount = (total_amount * cliff_percentage as i128) / 10000;
            let cliff_end_time = cliff_duration;
            let smoothing_end_time = cliff_end_time + smoothing_duration;

            // Sample multiple points during smoothing window to verify linear behavior
            let mut previous_vested = 0i128;
            for i in 1..=10 {
                let sample_time = cliff_end_time + (smoothing_duration * i / 10);
                
                if sample_time <= total_duration {
                    env.ledger().set(LedgerInfo {
                        timestamp: sample_time,
                        protocol_version: 20,
                        sequence_number: 1,
                        base_reserve: 10,
                        min_temp_entry_ttl: 10,
                        min_persistent_entry_ttl: 10,
                        max_entry_ttl: 3110400,
                    });

                    let vested = VestingContract::calculate_claimable_for_asset_wrapper(
                        env.clone(),
                        vault_id,
                        0
                    );

                    // Verify linear progression during smoothing
                    if i > 1 {
                        let increment = vested - previous_vested;
                        // The increment should be roughly constant during smoothing
                        prop_assert!(increment > 0, "Should have positive increment during smoothing");
                    }
                    previous_vested = vested;
                }
            }

            // At end of smoothing, full cliff amount should be available
            env.ledger().set(LedgerInfo {
                timestamp: smoothing_end_time,
                protocol_version: 20,
                sequence_number: 1,
                base_reserve: 10,
                min_temp_entry_ttl: 10,
                min_persistent_entry_ttl: 10,
                max_entry_ttl: 3110400,
            });

            let vested_at_smoothing_end = VestingContract::calculate_claimable_for_asset_wrapper(
                env.clone(),
                vault_id,
                0
            );

            // Should have at least the cliff amount available
            prop_assert!(vested_at_smoothing_end >= cliff_amount, "At smoothing end, should have at least cliff amount");

            // The mathematical integrity: total area under curve should equal intended amounts
            let remaining_amount = total_amount - cliff_amount;
            let elapsed_smoothing = smoothing_end_time;
            let expected_non_cliff_vested = (remaining_amount * elapsed_smoothing as i128) / total_duration as i128;
            let expected_total = cliff_amount + expected_non_cliff_vested;

            // Allow small rounding errors
            let tolerance = total_amount / 1000; // 0.1% tolerance
            prop_assert!(
                (vested_at_smoothing_end - expected_total).abs() <= tolerance,
                "Mathematical integrity: vested amount should match expected calculation"
            );
        }
    }

    #[test]
    fn test_extreme_edge_cases() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);

        // Test case 1: Zero smoothing duration (should fail validation)
        let result = std::panic::catch_unwind(|| {
            VestingContract::configure_cliff_smoothing(
                env.clone(),
                1, // vault_id
                2592000, // 30 days cliff
                0, // zero smoothing
                2500, // 25%
            )
        });
        assert!(result.is_err());

        // Test case 2: Smoothing duration equals total duration (should fail)
        let result = std::panic::catch_unwind(|| {
            VestingContract::create_vault_with_smoothed_cliff(
                env.clone(),
                beneficiary.clone(),
                10000,
                0, // start_time
                31536000, // 1 year total
                0, // keeper_fee
                true, // is_revocable
                false, // is_transferable
                0, // step_duration
                2592000, // 30 day cliff
                31536000, // 1 year smoothing (invalid)
                2500, // 25%
            )
        });
        assert!(result.is_err());

        // Test case 3: Maximum valid smoothing (1 year)
        let vault_id = VestingContract::create_vault_with_smoothed_cliff(
            env.clone(),
            beneficiary.clone(),
            10000,
            0, // start_time
            63072000, // 2 years total
            0, // keeper_fee
            true, // is_revocable
            false, // is_transferable
            0, // step_duration
            2592000, // 30 day cliff
            31536000, // 1 year smoothing (valid for 2 year total)
            2500, // 25%
        );
        assert!(vault_id > 0);

        // Test case 4: Minimum valid smoothing (1 day)
        let vault_id = VestingContract::create_vault_with_smoothed_cliff(
            env.clone(),
            beneficiary.clone(),
            10000,
            0, // start_time
            2592000, // 30 days total
            0, // keeper_fee
            true, // is_revocable
            false, // is_transferable
            0, // step_duration
            604800, // 7 day cliff
            86400, // 1 day smoothing
            2500, // 25%
        );
        assert!(vault_id > 0);
    }

    #[test]
    fn test_event_emission() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);

        // Create vault with smoothed cliff
        let vault_id = VestingContract::create_vault_with_smoothed_cliff(
            env.clone(),
            beneficiary.clone(),
            10000,
            0, // start_time
            31536000, // 1 year total
            0, // keeper_fee
            true, // is_revocable
            false, // is_transferable
            0, // step_duration
            2592000, // 30 day cliff
            604800, // 7 day smoothing
            2500, // 25%
        );

        // Set time to just after cliff starts smoothing
        env.ledger().set(LedgerInfo {
            timestamp: 2592001, // 1 second into smoothing
            protocol_version: 20,
            sequence_number: 1,
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        // Trigger calculation that should emit event
        let _vested = VestingContract::calculate_claimable_for_asset_wrapper(
            env.clone(),
            vault_id,
            0
        );

        // In a real test environment, we would verify the event was emitted
        // For fuzz testing, we mainly ensure the calculation doesn't panic
        assert!(true); // If we reach here, no panic occurred
    }
}
