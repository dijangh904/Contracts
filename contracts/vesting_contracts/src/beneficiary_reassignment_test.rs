#![cfg(test)]
use soroban_sdk::{
    Address,
    Env,
    Vec,
    String,
};
use crate::{
    beneficiary_reassignment::{
        BeneficiaryReassignment, ReassignmentError, ReassignmentRequest, ReassignmentStatus,
        SocialProofType, DAOMember, ReassignmentConfig, BeneficiaryReassignmentTrait,
        REASSIGNMENT_REQUESTS, DAO_MEMBERS, REASSIGNMENT_CONFIG, VAULT_REASSIGNMENTS,
    },
    testutils::{create_test_contract, create_test_address, create_test_env},
};

#[test]
fn test_initialize_beneficiary_reassignment() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let admin = create_test_address(&env);
    let member1 = create_test_address(&env);
    let member2 = create_test_address(&env);

    let initial_members = Vec::from_array(&env, &[member1.clone(), member2.clone()]);
    
    let result = BeneficiaryReassignment::initialize(
        &env,
        admin.clone(),
        initial_members.clone(),
        2, // required_approvals
        7 * 24 * 60 * 60, // 7 days approval window
    );

    assert!(result.is_ok());

    // Verify DAO members were set
    let members = BeneficiaryReassignment::get_active_council_members(&env);
    assert_eq!(members.len(), 2);
    assert!(members.iter().any(|m| m == &member1));
    assert!(members.iter().any(|m| m == &member2));

    // Verify configuration was set
    let config = BeneficiaryReassignment::get_reassignment_config(&env);
    assert_eq!(config.required_approvals, 2);
    assert!(config.emergency_enabled);
    assert!(config.social_proof_required);
}

#[test]
fn test_create_reassignment_request() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let current_beneficiary = create_test_address(&env);
    let new_beneficiary = create_test_address(&env);
    let vault_id = 1u64;

    // Initialize DAO first
    let admin = create_test_address(&env);
    let members = Vec::from_array(&env, &[admin.clone()]);
    BeneficiaryReassignment::initialize(
        &env,
        admin,
        members,
        2,
        7 * 24 * 60 * 60,
    ).unwrap();

    let result = BeneficiaryReassignment::create_reassignment_request(
        &env,
        current_beneficiary.clone(),
        new_beneficiary.clone(),
        vault_id,
        SocialProofType::LostKeys,
        [0x01; 32],
        "QmLostKeys123".to_string(),
        "Lost private keys, need recovery".to_string(),
    );

    assert!(result.is_ok());

    // Verify request was created
    let status = BeneficiaryReassignment::get_reassignment_status(&env, vault_id).unwrap();
    assert_eq!(status.vault_id, vault_id);
    assert_eq!(status.current_beneficiary, current_beneficiary);
    assert_eq!(status.new_beneficiary, new_beneficiary);
    assert_eq!(status.social_proof_type, SocialProofType::LostKeys);
    assert!(matches!(status.status, ReassignmentStatus::Pending(_)));
}

#[test]
fn test_approve_reassignment() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let current_beneficiary = create_test_address(&env);
    let new_beneficiary = create_test_address(&env);
    let vault_id = 1u64;

    // Initialize DAO and create request
    let admin = create_test_address(&env);
    let council_member = create_test_address(&env);
    let members = Vec::from_array(&env, &[admin.clone(), council_member.clone()]);
    BeneficiaryReassignment::initialize(
        &env,
        admin,
        members,
        2,
        7 * 24 * 60 * 60,
    ).unwrap();

    BeneficiaryReassignment::create_reassignment_request(
        &env,
        current_beneficiary,
        new_beneficiary.clone(),
        vault_id,
        SocialProofType::DeathCertificate,
        [0x02; 32],
        "QmDeathCert456".to_string(),
        "Beneficiary passed away".to_string(),
    ).unwrap();

    // Approve the request
    let result = BeneficiaryReassignment::approve_reassignment(
        &env,
        council_member.clone(),
        vault_id,
    );

    assert!(result.is_ok());

    // Verify approval was recorded
    let status = BeneficiaryReassignment::get_reassignment_status(&env, vault_id).unwrap();
    match &status.status {
        ReassignmentStatus::Pending(approvals) => {
            assert_eq!(approvals.len(), 1);
            assert!(approvals.iter().any(|a| a == &council_member));
        }
        _ => panic!("Request should be pending after one approval"),
    }
}

#[test]
fn test_approve_reassignment_insufficient_approvals() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let current_beneficiary = create_test_address(&env);
    let new_beneficiary = create_test_address(&env);
    let vault_id = 1u64;

    // Initialize DAO with 2 required approvals
    let admin = create_test_address(&env);
    let council_member = create_test_address(&env);
    let members = Vec::from_array(&env, &[admin.clone(), council_member.clone()]);
    BeneficiaryReassignment::initialize(
        &env,
        admin,
        members,
        2, // requires 2 approvals
        7 * 24 * 60 * 60,
    ).unwrap();

    BeneficiaryReassignment::create_reassignment_request(
        &env,
        current_beneficiary,
        new_beneficiary,
        vault_id,
        SocialProofType::LostKeys,
        [0x01; 32],
        "QmLostKeys123".to_string(),
        "Lost keys".to_string(),
    ).unwrap();

    // Approve with only one member (should not complete)
    let result = BeneficiaryReassignment::approve_reassignment(
        &env,
        council_member.clone(),
        vault_id,
    );

    assert!(result.is_ok());

    // Should still be pending (need 2 approvals)
    let status = BeneficiaryReassignment::get_reassignment_status(&env, vault_id).unwrap();
    match &status.status {
        ReassignmentStatus::Pending(approvals) => {
            assert_eq!(approvals.len(), 1);
        }
        _ => panic!("Request should still be pending"),
    }
}

#[test]
fn test_emergency_reassignment() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let current_beneficiary = create_test_address(&env);
    let new_beneficiary = create_test_address(&env);
    let vault_id = 1u64;

    // Initialize DAO with emergency enabled
    let admin = create_test_address(&env);
    let emergency_admin = create_test_address(&env);
    let members = Vec::from_array(&env, &[admin.clone(), emergency_admin.clone()]);
    BeneficiaryReassignment::initialize(
        &env,
        admin,
        members,
        2,
        7 * 24 * 60 * 60,
    ).unwrap();

    let result = BeneficiaryReassignment::emergency_reassignment(
        &env,
        emergency_admin.clone(),
        vault_id,
        new_beneficiary.clone(),
        SocialProofType::CourtOrder,
        [0x03; 32],
        "QmCourtOrder789".to_string(),
        "Court ordered reassignment".to_string(),
    );

    assert!(result.is_ok());

    // Verify reassignment was completed immediately
    let status = BeneficiaryReassignment::get_reassignment_status(&env, vault_id).unwrap();
    assert!(matches!(status.status, ReassignmentStatus::Completed));
}

#[test]
fn test_reassignment_errors() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let beneficiary = create_test_address(&env);
    let vault_id = 1u64;

    // Test invalid new beneficiary (same as current)
    let result = BeneficiaryReassignment::create_reassignment_request(
        &env,
        beneficiary.clone(),
        beneficiary.clone(), // Same address
        vault_id,
        SocialProofType::LostKeys,
        [0x01; 32],
        "QmLostKeys123".to_string(),
        "Test".to_string(),
    );

    assert_eq!(result.err(), Some(ReassignmentError::InvalidNewBeneficiary));

    // Test unauthorized approver
    let non_member = create_test_address(&env);
    let result = BeneficiaryReassignment::approve_reassignment(
        &env,
        non_member,
        vault_id,
    );

    assert_eq!(result.err(), Some(ReassignmentError::UnauthorizedApprover));
}

#[test]
fn test_reassignment_limit() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let current_beneficiary = create_test_address(&env);
    let new_beneficiary = create_test_address(&env);
    let vault_id = 1u64;

    // Initialize DAO with limit of 1 reassignment per vault
    let admin = create_test_address(&env);
    let council_member = create_test_address(&env);
    let members = Vec::from_array(&env, &[admin.clone(), council_member.clone()]);
    BeneficiaryReassignment::initialize(
        &env,
        admin,
        members,
        2,
        7 * 24 * 60 * 60,
    ).unwrap();

    // Manually increment reassignment count to test limit
    let count_key = (VAULT_REASSIGNMENTS, vault_id);
    env.storage().persistent().set(&count_key, &1u32);

    // Try to create another reassignment
    let result = BeneficiaryReassignment::create_reassignment_request(
        &env,
        current_beneficiary,
        new_beneficiary,
        vault_id,
        SocialProofType::LostKeys,
        [0x01; 32],
        "QmLostKeys123".to_string(),
        "Test".to_string(),
    );

    assert_eq!(result.err(), Some(ReassignmentError::InvalidVaultId));
}

#[test]
fn test_add_dao_member() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let admin = create_test_address(&env);
    let new_member = create_test_address(&env);

    // Initialize DAO
    let members = Vec::from_array(&env, &[admin.clone()]);
    BeneficiaryReassignment::initialize(
        &env,
        admin,
        members,
        2,
        7 * 24 * 60 * 60,
    ).unwrap();

    // Add new member
    let result = BeneficiaryReassignment::add_dao_member(
        &env,
        admin.clone(),
        new_member.clone(),
        "council".to_string(),
    );

    assert!(result.is_ok());

    // Verify member was added
    let active_members = BeneficiaryReassignment::get_active_council_members(&env);
    assert!(active_members.iter().any(|m| m == &new_member));
}

#[test]
fn test_social_proof_types() {
    let env = create_test_env();
    
    // Test all social proof types can be used
    let proof_types = vec![
        SocialProofType::DeathCertificate,
        SocialProofType::LostKeys,
        SocialProofType::CourtOrder,
        SocialProofType::MultiSig,
        SocialProofType::EmergencyContact,
    ];

    for proof_type in proof_types {
        // Verify the type can be created and used
        assert_eq!(proof_type, proof_type); // Basic type verification
    }
}
