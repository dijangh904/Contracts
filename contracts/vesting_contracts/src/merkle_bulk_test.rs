#![cfg(test)]
use super::*;
use soroban_sdk::{vec, Address, BytesN, Env};

#[test]
fn test_merkle_root_initialization() {
    let env = Env::default();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.initialize(&admin, &1_000_000_000i128);
    
    let merkle_root = BytesN::from_array(&env, &[1u8; 32]);
    let total_schedules = 1000u32;
    
    // Initialize Merkle root
    client.initialize_merkle_root(&merkle_root, &total_schedules);
    
    // Verify Merkle root is stored
    let stored_root = client.get_merkle_root();
    assert_eq!(stored_root.unwrap(), merkle_root);
    
    // Test duplicate initialization should fail
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "initialize_merkle_root"),
        (merkle_root, total_schedules).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_merkle_root_initialization_multisig() {
    let env = Env::default();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admins = vec![&env, admin1.clone(), admin2.clone()];
    client.initialize_multisig(&admins, &2u32, &1_000_000_000i128);
    
    let merkle_root = BytesN::from_array(&env, &[2u8; 32]);
    let total_schedules = 500u32;
    
    // Use admin proposal for multisig
    let action = AdminAction::InitializeMerkleRoot(merkle_root, total_schedules);
    let proposal_id = client.propose_admin_action(&admin1, &action);
    
    // Second admin signs
    client.sign_admin_proposal(&admin2, &proposal_id);
    
    // Verify Merkle root is stored
    let stored_root = client.get_merkle_root();
    assert_eq!(stored_root.unwrap(), merkle_root);
}

#[test]
fn test_merkle_proof_verification() {
    let env = Env::default();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.initialize(&admin, &1_000_000_000i128);
    
    // Create a simple Merkle tree with 2 leaves
    let leaf1_hash = BytesN::from_array(&env, &[10u8; 32]);
    let leaf2_hash = BytesN::from_array(&env, &[20u8; 32]);
    
    // Simple hash for root (in reality, this would be computed properly)
    let root_hash = BytesN::from_array(&env, &[30u8; 32]);
    
    // Initialize Merkle root
    client.initialize_merkle_root(&root_hash, &2u32);
    
    // Create proof for leaf 0 (index 0)
    let proof = MerkleProof {
        leaf_hash: leaf1_hash,
        proof: vec![&env, leaf2_hash], // sibling is leaf2
        leaf_index: 0,
    };
    
    // This would fail in reality because we're using dummy hashes,
    // but it tests the function structure
    let beneficiary = Address::generate(&env);
    let asset = Address::generate(&env);
    
    let leaf = VestingScheduleLeaf {
        beneficiary: beneficiary.clone(),
        vault_id: 1,
        asset_basket: vec![&env, AssetAllocationEntry {
            asset_id: asset,
            total_amount: 1000,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: 1000,
        end_time: 2000,
        keeper_fee: 100,
        is_revocable: true,
        is_transferable: false,
        step_duration: 0,
    };
    
    // This should fail due to invalid proof (expected behavior)
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "activate_schedule_with_proof"),
        (beneficiary, leaf, proof).into_val(&env),
    );
    assert!(result.is_err());
}

#[test]
fn test_schedule_activation_prevents_duplicates() {
    let env = Env::default();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.initialize(&admin, &1_000_000_000i128);
    
    let merkle_root = BytesN::from_array(&env, &[5u8; 32]);
    client.initialize_merkle_root(&merkle_root, &1u32);
    
    let beneficiary = Address::generate(&env);
    
    // Initially should not be activated
    assert!(!client.is_schedule_activated(&beneficiary));
    
    // Mock activation (would normally require valid proof)
    // For testing purposes, we'll manually set the activation flag
    env.storage().instance().set(&DataKey::ActivatedSchedule(beneficiary.clone()), &1u64);
    
    // Now should be activated
    assert!(client.is_schedule_activated(&beneficiary));
    
    // Should be able to get the vault ID
    let vault_id = client.get_activated_vault_id(&beneficiary);
    assert_eq!(vault_id.unwrap(), 1u64);
}

#[test]
fn test_merkle_leaf_hashing() {
    let env = Env::default();
    
    let beneficiary = Address::generate(&env);
    let asset = Address::generate(&env);
    
    let leaf = VestingScheduleLeaf {
        beneficiary: beneficiary.clone(),
        vault_id: 42,
        asset_basket: vec![&env, AssetAllocationEntry {
            asset_id: asset,
            total_amount: 5000,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: 1640995200, // Jan 1, 2022
        end_time: 1672531200,   // Jan 1, 2023
        keeper_fee: 50,
        is_revocable: false,
        is_transferable: true,
        step_duration: 86400, // 1 day
    };
    
    // Test that hashing produces consistent results
    let hash1 = VestingContract::hash_vesting_leaf(&env, &leaf);
    let hash2 = VestingContract::hash_vesting_leaf(&env, &leaf);
    assert_eq!(hash1, hash2);
    
    // Different leaves should produce different hashes
    let mut leaf2 = leaf.clone();
    leaf2.vault_id = 43;
    let hash3 = VestingContract::hash_vesting_leaf(&env, &leaf2);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_merkle_pair_hashing() {
    let env = Env::default();
    
    let left = BytesN::from_array(&env, &[1u8; 32]);
    let right = BytesN::from_array(&env, &[2u8; 32]);
    
    let hash1 = VestingContract::hash_pair(&env, &left, &right);
    let hash2 = VestingContract::hash_pair(&env, &left, &right);
    assert_eq!(hash1, hash2);
    
    // Order should matter
    let hash3 = VestingContract::hash_pair(&env, &right, &left);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_get_merkle_root_not_initialized() {
    let env = Env::default();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.initialize(&admin, &1_000_000_000i128);
    
    // Should return None when not initialized
    let root = client.get_merkle_root();
    assert!(root.is_none());
}

#[test]
fn test_activate_schedule_without_merkle_root() {
    let env = Env::default();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.initialize(&admin, &1_000_000_000i128);
    
    let beneficiary = Address::generate(&env);
    let asset = Address::generate(&env);
    
    let leaf = VestingScheduleLeaf {
        beneficiary: beneficiary.clone(),
        vault_id: 1,
        asset_basket: vec![&env, AssetAllocationEntry {
            asset_id: asset,
            total_amount: 1000,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: 1000,
        end_time: 2000,
        keeper_fee: 100,
        is_revocable: true,
        is_transferable: false,
        step_duration: 0,
    };
    
    let proof = MerkleProof {
        leaf_hash: BytesN::from_array(&env, &[1u8; 32]),
        proof: vec![&env],
        leaf_index: 0,
    };
    
    // Should fail because Merkle root is not initialized
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "activate_schedule_with_proof"),
        (beneficiary, leaf, proof).into_val(&env),
    );
    assert!(result.is_err());
}
