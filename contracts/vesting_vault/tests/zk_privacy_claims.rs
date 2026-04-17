#![cfg(test)]

use soroban_sdk::{Address, Env, BytesN, Symbol, String, IntoVal, Val, Error, Vec};
use soroban_sdk::testutils::{Address as _, Ledger};
use vesting_vault::{VestingVault, VestingVaultClient};
use vesting_vault::types::{Nullifier, ZKClaimProof};

#[test]
fn test_create_commitment() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    let vesting_id = 1u32;
    let amount = 1000i128;
    let commitment_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    // Test creating a commitment
    client.create_commitment(&user, &vesting_id, &amount, &commitment_hash);
    
    // Verify the commitment exists
    let commitment = client.get_commitment_info(&commitment_hash);
    assert!(commitment.is_some());
    
    let retrieved_commitment = commitment.unwrap();
    assert_eq!(retrieved_commitment.vesting_id, vesting_id);
    assert_eq!(retrieved_commitment.amount, amount);
    assert!(!retrieved_commitment.is_used);
    
    // Test duplicate commitment creation should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "create_commitment"),
        (user.clone(), vesting_id, amount, commitment_hash).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_nullifier_prevention() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let nullifier = Nullifier { hash: BytesN::from_array(&env, &[2u8; 32]) };
    
    // Initially nullifier should not be used
    assert!(!client.is_nullifier_used_public(&nullifier));
}

#[test]
fn test_merkle_root_management() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let merkle_root = BytesN::from_array(&env, &[3u8; 32]);
    
    // Add a Merkle root
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Verify the Merkle root exists
    let roots = client.get_merkle_roots();
    assert!(roots.contains(merkle_root.clone()));
    
    // Test duplicate Merkle root should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "add_merkle_root_admin"),
        (admin.clone(), merkle_root).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_private_claim_flow() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let amount = 1000i128;
    let commitment_hash = BytesN::from_array(&env, &[4u8; 32]);
    let merkle_root = BytesN::from_array(&env, &[5u8; 32]);
    let nullifier_hash = BytesN::from_array(&env, &[6u8; 32]);
    let nullifier = Nullifier { hash: nullifier_hash.clone() };
    
    // Setup: Create commitment
    client.create_commitment(&user, &vesting_id, &amount, &commitment_hash);
    
    // Setup: Add Merkle root
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Create ZK proof
    let zk_proof = ZKClaimProof {
        commitment_hash: commitment_hash.clone(),
        nullifier_hash: nullifier_hash.clone(),
        merkle_root: merkle_root.clone(),
        proof_data: soroban_sdk::Bytes::new(&env),
    };
    
    // Execute private claim
    client.private_claim(&zk_proof, &nullifier, &amount);
    
    // Verify nullifier is now used
    assert!(client.is_nullifier_used_public(&nullifier));
    
    // Verify commitment is marked as used
    let commitment = client.get_commitment_info(&commitment_hash);
    assert!(commitment.is_some());
    assert!(commitment.unwrap().is_used);
}

#[test]
fn test_private_claim_double_spending_prevention() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    let admin = Address::generate(&env);
    let vesting_id = 1u32;
    let amount = 1000i128;
    let commitment_hash = BytesN::from_array(&env, &[7u8; 32]);
    let merkle_root = BytesN::from_array(&env, &[8u8; 32]);
    let nullifier_hash = BytesN::from_array(&env, &[9u8; 32]);
    let nullifier = Nullifier { hash: nullifier_hash.clone() };
    
    // Setup: Create commitment and add Merkle root
    client.create_commitment(&user, &vesting_id, &amount, &commitment_hash);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Create ZK proof
    let zk_proof = ZKClaimProof {
        commitment_hash: commitment_hash.clone(),
        nullifier_hash: nullifier_hash.clone(),
        merkle_root: merkle_root.clone(),
        proof_data: soroban_sdk::Bytes::new(&env),
    };
    
    // Execute first private claim
    client.private_claim(&zk_proof, &nullifier, &amount);
    
    // Attempt second claim with same nullifier should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "private_claim"),
        (zk_proof, nullifier, amount).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_emergency_pause_with_private_claims() {
    let env = Env::default();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    let admin = Address::generate(&env);
    let auditor1 = Address::generate(&env);
    let auditor2 = Address::generate(&env);
    let amount = 1000i128;
    let commitment_hash = BytesN::from_array(&env, &[19u8; 32]);
    let merkle_root = BytesN::from_array(&env, &[20u8; 32]);
    let nullifier_hash = BytesN::from_array(&env, &[21u8; 32]);
    let nullifier = Nullifier { hash: nullifier_hash.clone() };
    
    // Setup: Create commitment and add Merkle root
    client.create_commitment(&user, &1u32, &amount, &commitment_hash);
    client.add_merkle_root_admin(&admin, &merkle_root);
    
    // Initialize auditors and trigger emergency pause
    let mut auditors = Vec::new(&env);
    auditors.push_back(auditor1.clone());
    auditors.push_back(auditor2.clone());
    auditors.push_back(Address::generate(&env));
    client.initialize_auditors(&admin, &auditors);
    client.request_emergency_pause(&auditor1, &String::from_str(&env, "Test pause"));
    client.request_emergency_pause(&auditor2, &String::from_str(&env, "Test pause"));
    
    // Create ZK proof
    let zk_proof = ZKClaimProof {
        commitment_hash: commitment_hash.clone(),
        nullifier_hash: nullifier_hash.clone(),
        merkle_root: merkle_root.clone(),
        proof_data: soroban_sdk::Bytes::new(&env),
    };
    
    // Private claim should fail during emergency pause
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "private_claim"),
        (zk_proof, nullifier, amount).into_val(&env),
    );
    assert!(result.is_err());
}
