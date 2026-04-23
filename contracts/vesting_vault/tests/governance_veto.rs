#![cfg(test)]

use soroban_sdk::{Address, Env};
use vesting_vault::VestingVault;

#[test]
fn test_initialize_token_supply() {
    let env = Env::default();
    let admin = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    let supply_info = VestingVault::get_token_supply_info(env.clone());
    assert_eq!(supply_info.total_supply, total_supply);
    assert!(supply_info.last_updated > 0);
}

#[test]
fn test_update_token_supply() {
    let env = Env::default();
    let admin = Address::random(&env);
    
    let initial_supply = 1_000_000i128;
    let new_supply = 2_000_000i128;
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), initial_supply);
    VestingVault::update_token_supply(env.clone(), admin.clone(), new_supply);
    
    let supply_info = VestingVault::get_token_supply_info(env.clone());
    assert_eq!(supply_info.total_supply, new_supply);
}

#[test]
fn test_set_governance_veto_threshold() {
    let env = Env::default();
    let admin = Address::random(&env);
    
    let threshold = 10u32; // 10%
    
    VestingVault::set_governance_veto_threshold(env.clone(), admin.clone(), threshold);
    
    let current_threshold = VestingVault::get_governance_veto_threshold(env.clone());
    assert_eq!(current_threshold, threshold);
}

#[test]
#[should_panic(expected = "Threshold cannot exceed 100%")]
fn test_set_invalid_governance_veto_threshold() {
    let env = Env::default();
    let admin = Address::random(&env);
    
    VestingVault::set_governance_veto_threshold(env.clone(), admin.clone(), 150u32);
}

#[test]
fn test_request_beneficiary_reassignment_small_amount() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 40_000i128; // 4% - below 5% threshold
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone(),
        current_beneficiary.clone(),
        new_beneficiary.clone(),
        1u32,
        reassignment_amount
    );
    
    let reassignment = VestingVault::get_beneficiary_reassignment(env.clone(), 1u32).unwrap();
    assert_eq!(reassignment.vesting_id, 1u32);
    assert_eq!(reassignment.current_beneficiary, current_beneficiary);
    assert_eq!(reassignment.new_beneficiary, new_beneficiary);
    assert_eq!(reassignment.total_amount, reassignment_amount);
    assert!(!reassignment.requires_governance_veto);
    assert!(!reassignment.is_executed);
    
    // Should have 48-hour timelock (not 7-day)
    let expected_effective_at = reassignment.requested_at + 172_800; // 48 hours
    assert_eq!(reassignment.effective_at, expected_effective_at);
}

#[test]
fn test_request_beneficiary_reassignment_large_amount() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 60_000i128; // 6% - above 5% threshold
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone(),
        current_beneficiary.clone(),
        new_beneficiary.clone(),
        1u32,
        reassignment_amount
    );
    
    let reassignment = VestingVault::get_beneficiary_reassignment(env.clone(), 1u32).unwrap();
    assert_eq!(reassignment.total_amount, reassignment_amount);
    assert!(reassignment.requires_governance_veto);
    assert!(!reassignment.is_executed);
    
    // Should have 7-day timelock
    let expected_effective_at = reassignment.requested_at + 604_800; // 7 days
    assert_eq!(reassignment.effective_at, expected_effective_at);
}

#[test]
fn test_execute_beneficiary_reassignment_small_amount() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 40_000i128; // 4% - below 5% threshold
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone(),
        current_beneficiary.clone(),
        new_beneficiary.clone(),
        1u32,
        reassignment_amount
    );
    
    // Advance time past 48-hour timelock
    env.ledger().set_timestamp(env.ledger().timestamp() + 172_801);
    
    VestingVault::execute_beneficiary_reassignment(env.clone(), 1u32);
    
    let reassignment = VestingVault::get_beneficiary_reassignment(env.clone(), 1u32).unwrap();
    assert!(reassignment.is_executed);
}

#[test]
#[should_panic(expected = "Timelock period has not expired")]
fn test_execute_beneficiary_reassignment_before_timelock() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 40_000i128; // 4% - below 5% threshold
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone,
        current_beneficiary,
        new_beneficiary,
        1u32,
        reassignment_amount
    );
    
    // Try to execute before timelock expires
    VestingVault::execute_beneficiary_reassignment(env.clone(), 1u32);
}

#[test]
fn test_cast_veto_vote() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    let voter = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 60_000i128; // 6% - above 5% threshold
    let voting_power = 30_000i128; // 3% of total supply
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone,
        current_beneficiary,
        new_beneficiary,
        1u32,
        reassignment_amount
    );
    
    VestingVault::cast_veto_vote(
        env.clone,
        voter.clone(),
        1u32,
        true, // vote for veto
        voting_power
    );
    
    let votes = VestingVault::get_veto_votes(env.clone(), 1u32);
    assert_eq!(votes.len(), 1);
    
    let vote = votes.get(0).unwrap();
    assert_eq!(vote.voter, voter);
    assert_eq!(vote.reassignment_id, 1u32);
    assert!(vote.vote_for_veto);
    assert_eq!(vote.voting_power, voting_power);
}

#[test]
#[should_panic(expected = "This reassignment does not require governance veto")]
fn test_cast_veto_vote_small_reassignment() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    let voter = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 40_000i128; // 4% - below 5% threshold
    let voting_power = 30_000i128;
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone,
        current_beneficiary,
        new_beneficiary,
        1u32,
        reassignment_amount
    );
    
    // Try to cast veto vote on small reassignment
    VestingVault::cast_veto_vote(
        env.clone,
        voter,
        1u32,
        true,
        voting_power
    );
}

#[test]
#[should_panic(expected = "Voter has already cast a vote")]
fn test_cast_duplicate_veto_vote() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    let voter = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 60_000i128; // 6% - above 5% threshold
    let voting_power = 30_000i128;
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone,
        current_beneficiary,
        new_beneficiary,
        1u32,
        reassignment_amount
    );
    
    VestingVault::cast_veto_vote(
        env.clone(),
        voter.clone(),
        1u32,
        true,
        voting_power
    );
    
    // Try to vote again
    VestingVault::cast_veto_vote(
        env.clone,
        voter,
        1u32,
        false, // vote against veto
        voting_power
    );
}

#[test]
fn test_successful_governance_veto() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    let voter1 = Address::random(&env);
    let voter2 = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 60_000i128; // 6% - above 5% threshold
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone,
        current_beneficiary,
        new_beneficiary,
        1u32,
        reassignment_amount
    );
    
    // Cast votes totaling 6% (above 5% threshold)
    VestingVault::cast_veto_vote(env.clone(), voter1.clone(), 1u32, true, 30_000i128); // 3%
    VestingVault::cast_veto_vote(env.clone(), voter2.clone(), 1u32, true, 30_000i128); // 3%
    
    // Reassignment should be vetoed and removed
    let reassignment = VestingVault::get_beneficiary_reassignment(env.clone(), 1u32);
    assert!(reassignment.is_none());
}

#[test]
fn test_failed_governance_veto() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    let voter = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 60_000i128; // 6% - above 5% threshold
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone,
        current_beneficiary,
        new_beneficiary,
        1u32,
        reassignment_amount
    );
    
    // Cast vote totaling 3% (below 5% threshold)
    VestingVault::cast_veto_vote(env.clone(), voter.clone(), 1u32, true, 30_000i128);
    
    // Advance time past 7-day timelock
    env.ledger().set_timestamp(env.ledger().timestamp() + 604_801);
    
    // Should be able to execute since veto threshold not reached
    VestingVault::execute_beneficiary_reassignment(env.clone(), 1u32);
    
    let reassignment = VestingVault::get_beneficiary_reassignment(env.clone(), 1u32).unwrap();
    assert!(reassignment.is_executed);
}

#[test]
fn test_requires_governance_veto_helper() {
    let env = Env::default();
    let admin = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    // Below threshold
    assert!(!VestingVault::requires_governance_veto(env.clone(), 40_000i128)); // 4%
    
    // At threshold
    assert!(!VestingVault::requires_governance_veto(env.clone(), 50_000i128)); // 5%
    
    // Above threshold
    assert!(VestingVault::requires_governance_veto(env.clone(), 60_000i128)); // 6%
}

#[test]
fn test_get_veto_status() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    let voter = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 60_000i128; // 6% - above 5% threshold
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone,
        current_beneficiary,
        new_beneficiary,
        1u32,
        reassignment_amount
    );
    
    // Initial status - no votes
    let (is_vetoed, veto_power, threshold) = VestingVault::get_veto_status(env.clone(), 1u32);
    assert!(!is_vetoed);
    assert_eq!(veto_power, 0);
    assert_eq!(threshold, 50_000i128); // 5% of 1M
    
    // Cast vote
    VestingVault::cast_veto_vote(env.clone(), voter.clone(), 1u32, true, 30_000i128);
    
    // Updated status - partial veto power
    let (is_vetoed, veto_power, threshold) = VestingVault::get_veto_status(env.clone(), 1u32);
    assert!(!is_vetoed);
    assert_eq!(veto_power, 30_000i128);
    assert_eq!(threshold, 50_000i128);
}

#[test]
fn test_multiple_reassignments() {
    let env = Env::default();
    let admin = Address::random(&env);
    let beneficiary1 = Address::random(&env);
    let beneficiary2 = Address::random(&env);
    let new_beneficiary1 = Address::random(&env);
    let new_beneficiary2 = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    // Request two reassignments
    VestingVault::request_beneficiary_reassignment(
        env.clone(),
        beneficiary1.clone(),
        new_beneficiary1.clone(),
        1u32,
        40_000i128 // 4% - no veto required
    );
    
    VestingVault::request_beneficiary_reassignment(
        env.clone(),
        beneficiary2.clone(),
        new_beneficiary2.clone(),
        2u32,
        60_000i128 // 6% - veto required
    );
    
    let reassignment1 = VestingVault::get_beneficiary_reassignment(env.clone(), 1u32).unwrap();
    let reassignment2 = VestingVault::get_beneficiary_reassignment(env.clone(), 2u32).unwrap();
    
    assert!(!reassignment1.requires_governance_veto);
    assert!(reassignment2.requires_governance_veto);
    
    assert_eq!(reassignment1.reassignment_id, 1u32);
    assert_eq!(reassignment2.reassignment_id, 2u32);
}

#[test]
#[should_panic(expected = "Veto period has expired")]
fn test_cast_vote_after_deadline() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    let voter = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 60_000i128; // 6% - above 5% threshold
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone,
        current_beneficiary,
        new_beneficiary,
        1u32,
        reassignment_amount
    );
    
    // Advance time past 7-day deadline
    env.ledger().set_timestamp(env.ledger().timestamp() + 604_801);
    
    // Try to vote after deadline
    VestingVault::cast_veto_vote(env.clone, voter, 1u32, true, 30_000i128);
}

#[test]
#[should_panic(expected = "Reassignment vetoed by governance")]
fn test_execute_vetoed_reassignment() {
    let env = Env::default();
    let admin = Address::random(&env);
    let current_beneficiary = Address::random(&env);
    let new_beneficiary = Address::random(&env);
    let voter1 = Address::random(&env);
    let voter2 = Address::random(&env);
    
    let total_supply = 1_000_000i128;
    let reassignment_amount = 60_000i128; // 6% - above 5% threshold
    
    VestingVault::initialize_token_supply(env.clone(), admin.clone(), total_supply);
    
    VestingVault::request_beneficiary_reassignment(
        env.clone,
        current_beneficiary,
        new_beneficiary,
        1u32,
        reassignment_amount
    );
    
    // Cast sufficient votes to veto
    VestingVault::cast_veto_vote(env.clone(), voter1.clone(), 1u32, true, 30_000i128);
    VestingVault::cast_veto_vote(env.clone(), voter2.clone(), 1u32, true, 30_000i128);
    
    // Try to execute after timelock (should fail due to veto)
    env.ledger().set_timestamp(env.ledger().timestamp() + 604_801);
    VestingVault::execute_beneficiary_reassignment(env.clone(), 1u32);
}
