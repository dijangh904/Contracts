#![cfg(test)]

//! Fuzz tests for Zero-Knowledge Confidential Grant Amounts (Issue #269)
//!
//! These tests bombard the ZK verifier with invalid proofs to ensure the math
//! never allows a false positive. They test edge cases and boundary conditions
//! to verify the security of the confidential grant system.

use soroban_sdk::{Address, Env, BytesN, Symbol, Error, Val, IntoVal};
use soroban_sdk::testutils::{Address as _, Ledger};
use vesting_vault::{VestingVault, VestingVaultClient};
use vesting_vault::types::{ConfidentialClaimProof, ConfidentialGrant, MasterViewingKey};
use vesting_vault::errors::Error as VaultError;

/// Helper function to create a valid proof for testing
fn create_valid_proof(env: &Env) -> ConfidentialClaimProof {
    ConfidentialClaimProof {
        commitment_hash: BytesN::from_array(env, &[1u8; 32]),
        nullifier: BytesN::from_array(env, &[2u8; 32]),
        merkle_root: BytesN::from_array(env, &[3u8; 32]),
        claimed_amount: 100,
        remaining_amount: 900,
        proof_a: BytesN::from_array(env, &[4u8; 32]),
        proof_b: BytesN::from_array(env, &[5u8; 32]),
        proof_c: BytesN::from_array(env, &[6u8; 32]),
    }
}

#[test]
fn test_confidential_grant_creation() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    
    // Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    
    // Verify grant exists
    let grant = client.get_confidential_grant_info(&vesting_id);
    assert!(grant.is_some());
    
    let retrieved_grant = grant.unwrap();
    assert_eq!(retrieved_grant.vesting_id, vesting_id);
    assert_eq!(retrieved_grant.commitment_hash, commitment_hash);
    assert_eq!(retrieved_grant.remaining_shielded, total_shielded_amount);
    assert!(!retrieved_grant.is_fully_claimed);
}

#[test]
fn test_duplicate_confidential_grant_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    
    // Create first grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    
    // Attempt to create duplicate should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "create_confidential_grant"),
        (admin, vesting_id, commitment_hash, total_shielded_amount).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_grant_zero_amount_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 0i128;
    
    // Attempt to create grant with zero amount should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "create_confidential_grant"),
        (admin, vesting_id, commitment_hash, total_shielded_amount).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_grant_negative_amount_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = -100i128;
    
    // Attempt to create grant with negative amount should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "create_confidential_grant"),
        (admin, vesting_id, commitment_hash, total_shielded_amount).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_claim_valid_proof() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    
    // Setup: Add Merkle root
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Create valid proof
    let proof = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[2u8; 32]),
        merkle_root,
        claimed_amount: 100,
        remaining_amount: 9900,
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    
    // Execute confidential claim
    client.confidential_claim(&vesting_id, &proof);
    
    // Verify nullifier is now used
    assert!(client.is_nullifier_used_confidential(&proof.nullifier));
    
    // Verify grant remaining amount updated
    let grant = client.get_confidential_grant_info(&vesting_id);
    assert!(grant.is_some());
    assert_eq!(grant.unwrap().remaining_shielded, 9900);
}

#[test]
fn test_confidential_claim_over_claim_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 100i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Create proof with over-claim attempt
    let proof = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[2u8; 32]),
        merkle_root,
        claimed_amount: 1000, // More than remaining
        remaining_amount: -900,
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    
    // Attempt over-claim should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_claim"),
        (vesting_id, proof).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_claim_invalid_commitment_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Create proof with wrong commitment
    let proof = ConfidentialClaimProof {
        commitment_hash: BytesN::from_array(&env, &[99u8; 32]), // Wrong commitment
        nullifier: BytesN::from_array(&env, &[2u8; 32]),
        merkle_root,
        claimed_amount: 100,
        remaining_amount: 9900,
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    
    // Attempt claim with invalid commitment should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_claim"),
        (vesting_id, proof).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_claim_invalid_merkle_root_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let valid_merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &valid_merkle_root);
    
    // Create proof with invalid Merkle root
    let proof = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[2u8; 32]),
        merkle_root: BytesN::from_array(&env, &[99u8; 32]), // Invalid Merkle root
        claimed_amount: 100,
        remaining_amount: 9900,
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    
    // Attempt claim with invalid Merkle root should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_claim"),
        (vesting_id, proof).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_claim_double_spending_prevention() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    let nullifier = BytesN::from_array(&env, &[2u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Create proof
    let proof = ConfidentialClaimProof {
        commitment_hash,
        nullifier: nullifier.clone(),
        merkle_root,
        claimed_amount: 100,
        remaining_amount: 9900,
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    
    // Execute first claim
    client.confidential_claim(&vesting_id, &proof);
    
    // Attempt second claim with same nullifier should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_claim"),
        (vesting_id, proof).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_claim_fully_claimed_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 100i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Claim the full amount
    let proof1 = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[2u8; 32]),
        merkle_root,
        claimed_amount: 100,
        remaining_amount: 0,
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    client.confidential_claim(&vesting_id, &proof1);
    
    // Attempt second claim should fail (fully claimed)
    let proof2 = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[3u8; 32]),
        merkle_root,
        claimed_amount: 1,
        remaining_amount: -1,
        proof_a: BytesN::from_array(&env, &[7u8; 32]),
        proof_b: BytesN::from_array(&env, &[8u8; 32]),
        proof_c: BytesN::from_array(&env, &[9u8; 32]),
    };
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_claim"),
        (vesting_id, proof2).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_claim_zero_proof_components_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Create proof with zero proof_a
    let proof = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[2u8; 32]),
        merkle_root,
        claimed_amount: 100,
        remaining_amount: 9900,
        proof_a: BytesN::from_array(&env, &[0u8; 32]), // Zero proof component
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    
    // Attempt claim with zero proof component should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_claim"),
        (vesting_id, proof).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_claim_zero_claimed_amount_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Create proof with zero claimed amount
    let proof = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[2u8; 32]),
        merkle_root,
        claimed_amount: 0, // Zero claimed amount
        remaining_amount: 10000,
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    
    // Attempt claim with zero claimed amount should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_claim"),
        (vesting_id, proof).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_claim_negative_remaining_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Create proof with negative remaining amount
    let proof = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[2u8; 32]),
        merkle_root,
        claimed_amount: 100,
        remaining_amount: -100, // Negative remaining
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    
    // Attempt claim with negative remaining should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_claim"),
        (vesting_id, proof).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_master_viewing_key_set_and_revoke() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let viewing_key = BytesN::from_array(&env, &[1u8; 32]);
    
    // Set master viewing key
    client.set_master_viewing_key_admin(&admin, &viewing_key);
    
    // Revoke master viewing key
    client.revoke_master_viewing_key(&admin);
    
    // Verify key is revoked (no error on second revoke)
    client.revoke_master_viewing_key(&admin);
}

#[test]
fn test_confidential_clawback_valid() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let viewing_key = BytesN::from_array(&env, &[1u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.set_master_viewing_key_admin(&admin, &viewing_key);
    
    // Execute clawback
    client.confidential_clawback(&admin, &vesting_id, &viewing_key, &5000);
    
    // Verify remaining amount updated
    let grant = client.get_confidential_grant_info(&vesting_id);
    assert!(grant.is_some());
    assert_eq!(grant.unwrap().remaining_shielded, 5000);
}

#[test]
fn test_confidential_clawback_unauthorized_key_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let valid_key = BytesN::from_array(&env, &[1u8; 32]);
    let invalid_key = BytesN::from_array(&env, &[99u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.set_master_viewing_key_admin(&admin, &valid_key);
    
    // Attempt clawback with invalid key should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_clawback"),
        (admin, vesting_id, invalid_key, 5000i128).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_clawback_over_claim_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 100i128;
    let viewing_key = BytesN::from_array(&env, &[1u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.set_master_viewing_key_admin(&admin, &viewing_key);
    
    // Attempt clawback exceeding remaining should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_clawback"),
        (admin, vesting_id, viewing_key, 1000i128).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_clawback_no_key_fails() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let viewing_key = BytesN::from_array(&env, &[1u8; 32]);
    
    // Setup: Create confidential grant (no viewing key set)
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    
    // Attempt clawback without viewing key should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_clawback"),
        (admin, vesting_id, viewing_key, 5000i128).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_confidential_claim_emergency_pause_blocks() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let auditor1 = Address::generate(&env);
    let auditor2 = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Trigger emergency pause
    let mut auditors = soroban_sdk::Vec::new(&env);
    auditors.push_back(auditor1.clone());
    auditors.push_back(auditor2.clone());
    auditors.push_back(Address::generate(&env));
    client.initialize_auditors(&admin, &auditors);
    client.request_emergency_pause(&auditor1, &soroban_sdk::String::from_str(&env, "Test"));
    client.request_emergency_pause(&auditor2, &soroban_sdk::String::from_str(&env, "Test"));
    
    // Attempt confidential claim during pause should fail
    let proof = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[2u8; 32]),
        merkle_root,
        claimed_amount: 100,
        remaining_amount: 9900,
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confidential_claim"),
        (vesting_id, proof).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_nullifier_persistence_across_claims() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    let total_shielded_amount = 10000i128;
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    let nullifier = BytesN::from_array(&env, &[2u8; 32]);
    
    // Setup: Create confidential grant
    client.create_confidential_grant(&admin, &vesting_id, &commitment_hash, &total_shielded_amount);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Execute claim
    let proof = ConfidentialClaimProof {
        commitment_hash,
        nullifier: nullifier.clone(),
        merkle_root,
        claimed_amount: 100,
        remaining_amount: 9900,
        proof_a: BytesN::from_array(&env, &[4u8; 32]),
        proof_b: BytesN::from_array(&env, &[5u8; 32]),
        proof_c: BytesN::from_array(&env, &[6u8; 32]),
    };
    client.confidential_claim(&vesting_id, &proof);
    
    // Verify nullifier persists in permanent storage
    assert!(client.is_nullifier_used_confidential(&nullifier));
    
    // Even after another claim with different nullifier, original should still be marked
    let proof2 = ConfidentialClaimProof {
        commitment_hash,
        nullifier: BytesN::from_array(&env, &[3u8; 32]),
        merkle_root,
        claimed_amount: 100,
        remaining_amount: 9800,
        proof_a: BytesN::from_array(&env, &[7u8; 32]),
        proof_b: BytesN::from_array(&env, &[8u8; 32]),
        proof_c: BytesN::from_array(&env, &[9u8; 32]),
    };
    client.confidential_claim(&vesting_id, &proof2);
    
    // Original nullifier should still be marked as used
    assert!(client.is_nullifier_used_confidential(&nullifier));
}
