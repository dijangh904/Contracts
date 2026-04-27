// src/test_vesting_stress.rs

#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

// Assume your contract client looks like this
use crate::VestingContractClient;

#[test]
fn stress_test_batch_claim_500_schedules() {
    let env = Env::default();

    // 🔧 Extend ledger so nothing expires during test
    env.ledger().with_mut(|li| {
        li.timestamp = 1_000_000;
        li.sequence_number = 1;
    });

    let contract_id = env.register_contract(None, crate::VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // -----------------------------------------
    // STEP 1: Create 500 vesting schedules
    // -----------------------------------------
    let total_schedules = 500u32;

    for i in 0..total_schedules {
        client.create_vesting(
            &user,
            &1000_i128,                // amount
            &(1_000_000 + i as u64),  // start time
            &100,                     // duration
        );
    }

    // -----------------------------------------
    // STEP 2: Move time forward (so claims are valid)
    // -----------------------------------------
    env.ledger().with_mut(|li| {
        li.timestamp = 2_000_000;
        li.sequence_number += 1;
    });

    // -----------------------------------------
    // STEP 3: Measure CPU / instruction usage
    // -----------------------------------------
    // Soroban doesn't expose exact CPU count directly in tests,
    // but we can detect failures caused by budget exhaustion.

    let result = std::panic::catch_unwind(|| {
        client.batch_claim(&user);
    });

    // -----------------------------------------
    // STEP 4: Assert no panic (no budget exceeded)
    // -----------------------------------------
    assert!(
        result.is_ok(),
        "batch_claim exceeded Soroban CPU budget or panicked under stress"
    );

    // -----------------------------------------
    // STEP 5: Validate correctness
    // -----------------------------------------
    let remaining = client.get_active_schedules(&user);

    assert_eq!(
        remaining.len(),
        0,
        "All vesting schedules should be claimed"
    );
}