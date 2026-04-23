use soroban_sdk::{contractimpl, Address, Env};
use vesting_vault::VestingVault;

pub struct GovernanceVetoExample;

#[contractimpl]
impl GovernanceVetoExample {
    /// Example demonstrating the complete governance veto flow
    pub fn demonstrate_governance_veto_flow(e: Env) {
        // Setup participants
        let admin = Address::random(&e);
        let current_beneficiary = Address::random(&e);
        let new_beneficiary = Address::random(&e);
        let voter1 = Address::random(&e);
        let voter2 = Address::random(&e);
        let voter3 = Address::random(&e);
        
        let total_supply = 1_000_000i128;
        
        // Step 1: Initialize token supply for governance calculations
        VestingVault::initialize_token_supply(e.clone(), admin.clone(), total_supply);
        
        // Step 2: Set custom veto threshold (optional)
        VestingVault::set_governance_veto_threshold(e.clone(), admin.clone(), 5u32); // 5%
        
        // Step 3: Request large beneficiary reassignment (>5% threshold)
        let vesting_id = 1u32;
        let reassignment_amount = 60_000i128; // 6% of total supply
        
        VestingVault::request_beneficiary_reassignment(
            e.clone(),
            current_beneficiary.clone(),
            new_beneficiary.clone(),
            vesting_id,
            reassignment_amount
        );
        
        // Step 4: Verify reassignment details
        let reassignment = VestingVault::get_beneficiary_reassignment(e.clone(), 1u32).unwrap();
        assert_eq!(reassignment.vesting_id, vesting_id);
        assert_eq!(reassignment.total_amount, reassignment_amount);
        assert!(reassignment.requires_governance_veto);
        assert!(!reassignment.is_executed);
        
        // Step 5: Check if governance veto is required
        let requires_veto = VestingVault::requires_governance_veto(e.clone(), reassignment_amount);
        assert!(requires_veto);
        
        // Step 6: Token holders cast votes during 7-day period
        let current_time = e.ledger().timestamp();
        
        // Voter1 casts veto vote (3% of supply)
        VestingVault::cast_veto_vote(
            e.clone(),
            voter1.clone(),
            1u32,
            true, // vote for veto
            30_000i128
        );
        
        // Voter2 casts veto vote (2% of supply)
        VestingVault::cast_veto_vote(
            e.clone(),
            voter2.clone(),
            1u32,
            true, // vote for veto
            20_000i128
        );
        
        // Voter3 casts vote against veto (1% of supply)
        VestingVault::cast_veto_vote(
            e.clone(),
            voter3.clone(),
            1u32,
            false, // vote against veto
            10_000i128
        );
        
        // Step 7: Check veto status
        let (is_vetoed, veto_power, threshold) = VestingVault::get_veto_status(e.clone(), 1u32);
        assert!(is_vetoed); // 5% veto power >= 5% threshold
        assert_eq!(veto_power, 50_000i128); // 30k + 20k
        assert_eq!(threshold, 50_000i128); // 5% of 1M
        
        // Step 8: Verify reassignment was cancelled due to veto
        let cancelled_reassignment = VestingVault::get_beneficiary_reassignment(e.clone(), 1u32);
        assert!(cancelled_reassignment.is_none());
        
        // Step 9: Check voting history
        let votes = VestingVault::get_veto_votes(e.clone(), 1u32);
        assert_eq!(votes.len(), 3);
    }
    
    /// Example demonstrating small reassignment without governance veto
    pub fn demonstrate_small_reassignment_flow(e: Env) {
        // Setup participants
        let admin = Address::random(&e);
        let current_beneficiary = Address::random(&e);
        let new_beneficiary = Address::random(&e);
        
        let total_supply = 1_000_000i128;
        
        // Initialize token supply
        VestingVault::initialize_token_supply(e.clone(), admin.clone(), total_supply);
        
        // Request small reassignment (below 5% threshold)
        let vesting_id = 2u32;
        let reassignment_amount = 40_000i128; // 4% of total supply
        
        VestingVault::request_beneficiary_reassignment(
            e.clone(),
            current_beneficiary.clone(),
            new_beneficiary.clone(),
            vesting_id,
            reassignment_amount
        );
        
        // Verify reassignment details
        let reassignment = VestingVault::get_beneficiary_reassignment(e.clone(), 2u32).unwrap();
        assert_eq!(reassignment.total_amount, reassignment_amount);
        assert!(!reassignment.requires_governance_veto);
        assert!(!reassignment.is_executed);
        
        // Check if governance veto is required
        let requires_veto = VestingVault::requires_governance_veto(e.clone(), reassignment_amount);
        assert!(!requires_veto);
        
        // Wait 48-hour timelock (small reassignment)
        let current_time = e.ledger().timestamp();
        e.ledger().set_timestamp(current_time + 172_801); // 48 hours + 1 second
        
        // Execute reassignment
        VestingVault::execute_beneficiary_reassignment(e.clone(), 2u32);
        
        // Verify successful execution
        let executed_reassignment = VestingVault::get_beneficiary_reassignment(e.clone(), 2u32).unwrap();
        assert!(executed_reassignment.is_executed);
    }
    
    /// Example demonstrating failed veto (reassignment proceeds)
    pub fn demonstrate_failed_veto_flow(e: Env) {
        // Setup participants
        let admin = Address::random(&e);
        let current_beneficiary = Address::random(&e);
        let new_beneficiary = Address::random(&e);
        let voter1 = Address::random(&e);
        let voter2 = Address::random(&e);
        
        let total_supply = 1_000_000i128;
        
        // Initialize token supply
        VestingVault::initialize_token_supply(e.clone(), admin.clone(), total_supply);
        
        // Request large reassignment
        let vesting_id = 3u32;
        let reassignment_amount = 60_000i128; // 6% of total supply
        
        VestingVault::request_beneficiary_reassignment(
            e.clone,
            current_beneficiary.clone(),
            new_beneficiary.clone(),
            vesting_id,
            reassignment_amount
        );
        
        // Cast insufficient veto votes (only 4% total, below 5% threshold)
        VestingVault::cast_veto_vote(e.clone(), voter1.clone(), 3u32, true, 25_000i128); // 2.5%
        VestingVault::cast_veto_vote(e.clone(), voter2.clone(), 3u32, true, 15_000i128); // 1.5%
        
        // Check veto status (should not be vetoed)
        let (is_vetoed, veto_power, threshold) = VestingVault::get_veto_status(e.clone(), 3u32);
        assert!(!is_vetoed);
        assert_eq!(veto_power, 40_000i128); // 25k + 15k
        assert_eq!(threshold, 50_000i128); // 5% of 1M
        
        // Wait 7-day timelock
        let current_time = e.ledger().timestamp();
        e.ledger().set_timestamp(current_time + 604_801); // 7 days + 1 second
        
        // Execute reassignment (should succeed)
        VestingVault::execute_beneficiary_reassignment(e.clone(), 3u32);
        
        // Verify successful execution
        let executed_reassignment = VestingVault::get_beneficiary_reassignment(e.clone(), 3u32).unwrap();
        assert!(executed_reassignment.is_executed);
    }
    
    /// Example demonstrating multiple concurrent reassignments
    pub fn demonstrate_multiple_reassignments(e: Env) {
        // Setup participants
        let admin = Address::random(&e);
        let beneficiary1 = Address::random(&e);
        let beneficiary2 = Address::random(&e);
        let new_beneficiary1 = Address::random(&e);
        let new_beneficiary2 = Address::random(&e);
        let voter = Address::random(&e);
        
        let total_supply = 1_000_000i128;
        
        // Initialize token supply
        VestingVault::initialize_token_supply(e.clone(), admin.clone(), total_supply);
        
        // Request multiple reassignments simultaneously
        VestingVault::request_beneficiary_reassignment(
            e.clone(),
            beneficiary1.clone(),
            new_beneficiary1.clone(),
            4u32,
            40_000i128 // 4% - no veto required
        );
        
        VestingVault::request_beneficiary_reassignment(
            e.clone(),
            beneficiary2.clone(),
            new_beneficiary2.clone(),
            5u32,
            60_000i128 // 6% - veto required
        );
        
        // Verify both reassignments exist
        let reassignment1 = VestingVault::get_beneficiary_reassignment(e.clone(), 4u32).unwrap();
        let reassignment2 = VestingVault::get_beneficiary_reassignment(e.clone(), 5u32).unwrap();
        
        assert!(!reassignment1.requires_governance_veto);
        assert!(reassignment2.requires_governance_veto);
        
        // Cast veto vote only for the large reassignment
        VestingVault::cast_veto_vote(e.clone(), voter.clone(), 5u32, true, 30_000i128);
        
        // Wait appropriate timelocks
        let current_time = e.ledger().timestamp();
        e.ledger().set_timestamp(current_time + 604_801); // 7 days (for large reassignment)
        
        // Execute small reassignment (should succeed)
        VestingVault::execute_beneficiary_reassignment(e.clone(), 4u32);
        let executed1 = VestingVault::get_beneficiary_reassignment(e.clone(), 4u32).unwrap();
        assert!(executed1.is_executed);
        
        // Large reassignment should still be pending (insufficient veto power)
        let pending2 = VestingVault::get_beneficiary_reassignment(e.clone(), 5u32).unwrap();
        assert!(!pending2.is_executed);
        
        // Add more veto power to reach threshold
        let voter2 = Address::random(&e);
        VestingVault::cast_veto_vote(e.clone(), voter2.clone(), 5u32, true, 25_000i128);
        
        // Now large reassignment should be vetoed
        let vetoed2 = VestingVault::get_beneficiary_reassignment(e.clone(), 5u32);
        assert!(vetoed2.is_none);
    }
    
    /// Example demonstrating threshold configuration
    pub fn demonstrate_threshold_configuration(e: Env) {
        // Setup participants
        let admin = Address::random(&e);
        let current_beneficiary = Address::random(&e);
        let new_beneficiary = Address::random(&e);
        
        let total_supply = 1_000_000i128;
        
        // Initialize token supply
        VestingVault::initialize_token_supply(e.clone(), admin.clone(), total_supply);
        
        // Check default threshold (should be 5%)
        let default_threshold = VestingVault::get_governance_veto_threshold(e.clone());
        assert_eq!(default_threshold, 5u32);
        
        // Set custom threshold to 3%
        VestingVault::set_governance_veto_threshold(e.clone(), admin.clone(), 3u32);
        
        // Verify new threshold
        let new_threshold = VestingVault::get_governance_veto_threshold(e.clone());
        assert_eq!(new_threshold, 3u32);
        
        // Test with new threshold
        let reassignment_amount = 40_000i128; // 4% of total supply
        
        // With 3% threshold, this should require veto
        let requires_veto = VestingVault::requires_governance_veto(e.clone(), reassignment_amount);
        assert!(requires_veto);
        
        // Request reassignment
        VestingVault::request_beneficiary_reassignment(
            e.clone(),
            current_beneficiary.clone(),
            new_beneficiary.clone(),
            6u32,
            reassignment_amount
        );
        
        // Verify it requires governance veto
        let reassignment = VestingVault::get_beneficiary_reassignment(e.clone(), 6u32).unwrap();
        assert!(reassignment.requires_governance_veto);
        
        // Should have 7-day timelock due to exceeding 3% threshold
        let expected_effective_at = reassignment.requested_at + 604_800; // 7 days
        assert_eq!(reassignment.effective_at, expected_effective_at);
    }
    
    /// Example demonstrating token supply updates
    pub fn demonstrate_token_supply_updates(e: Env) {
        // Setup participants
        let admin = Address::random(&e);
        let current_beneficiary = Address::random(&e);
        let new_beneficiary = Address::random(&e);
        
        let initial_supply = 1_000_000i128;
        
        // Initialize with initial supply
        VestingVault::initialize_token_supply(e.clone(), admin.clone(), initial_supply);
        
        // Check initial state
        let supply_info = VestingVault::get_token_supply_info(e.clone());
        assert_eq!(supply_info.total_supply, initial_supply);
        
        // Test threshold with initial supply
        let reassignment_amount = 60_000i128; // 6% of initial supply
        let requires_veto_initial = VestingVault::requires_governance_veto(e.clone(), reassignment_amount);
        assert!(requires_veto_initial);
        
        // Update token supply (e.g., due to token minting/burning)
        let new_supply = 2_000_000i128;
        VestingVault::update_token_supply(e.clone(), admin.clone(), new_supply);
        
        // Verify updated supply
        let updated_supply_info = VestingVault::get_token_supply_info(e.clone());
        assert_eq!(updated_supply_info.total_supply, new_supply);
        
        // Same amount is now only 3% of new supply
        let requires_veto_updated = VestingVault::requires_governance_veto(e.clone(), reassignment_amount);
        assert!(!requires_veto_updated); // 60k is only 3% of 2M
        
        // Request reassignment with updated supply
        VestingVault::request_beneficiary_reassignment(
            e.clone(),
            current_beneficiary.clone(),
            new_beneficiary.clone(),
            7u32,
            reassignment_amount
        );
        
        // Should not require governance veto due to larger supply
        let reassignment = VestingVault::get_beneficiary_reassignment(e.clone(), 7u32).unwrap();
        assert!(!reassignment.requires_governance_veto);
        
        // Should have 48-hour timelock
        let expected_effective_at = reassignment.requested_at + 172_800; // 48 hours
        assert_eq!(reassignment.effective_at, expected_effective_at);
    }
}
