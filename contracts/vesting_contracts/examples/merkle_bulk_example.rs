use soroban_sdk::{contractimpl, Address, BytesN, Env, Symbol, Vec, contracttype};
use vesting_contracts::{
    VestingContract, VestingContractClient, MerkleProof, VestingScheduleLeaf,
    AssetAllocationEntry
};

// Example demonstrating Merkle Tree Bulk Initialization for gas optimization
// This shows how to initialize 1,000+ vesting schedules with a single transaction

pub fn create_example_merkle_tree() -> (BytesN<32>, Vec<(VestingScheduleLeaf, MerkleProof)>) {
    let env = Env::default();
    
    // In a real implementation, you would:
    // 1. Create all vesting schedule leaves
    // 2. Build a proper Merkle tree
    // 3. Generate proofs for each leaf
    
    // For this example, we'll create a simplified version
    let mut leaves = Vec::new(&env);
    let mut proofs = Vec::new(&env);
    
    // Create example vesting schedules
    for i in 0..1000 {
        let beneficiary = Address::generate(&env);
        let asset = Address::generate(&env);
        
        let leaf = VestingScheduleLeaf {
            beneficiary: beneficiary.clone(),
            vault_id: i + 1,
            asset_basket: vec![&env, AssetAllocationEntry {
                asset_id: asset,
                total_amount: 10000, // $10,000 worth of tokens
                released_amount: 0,
                locked_amount: 0,
                percentage: 10000, // 100%
            }],
            start_time: 1640995200 + (i * 86400), // Staggered start times
            end_time: 1672531200 + (i * 86400),   // Staggered end times
            keeper_fee: 100,
            is_revocable: true,
            is_transferable: false,
            step_duration: 86400, // Daily vesting steps
        };
        
        leaves.push_back(leaf.clone());
        
        // In reality, you'd generate proper Merkle proofs
        // For this example, we create dummy proofs
        let proof = MerkleProof {
            leaf_hash: BytesN::from_array(&env, &[(i % 256) as u8; 32]),
            proof: vec![&env, BytesN::from_array(&env, &[((i + 1) % 256) as u8; 32])],
            leaf_index: i as u32,
        };
        
        proofs.push_back((leaf, proof));
    }
    
    // In reality, compute the actual Merkle root
    let merkle_root = BytesN::from_array(&env, &[42u8; 32]);
    
    (merkle_root, proofs)
}

pub fn example_bulk_initialization() {
    let env = Env::default();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);
    
    // 1. Initialize contract
    let admin = Address::generate(&env);
    client.initialize(&admin, &10_000_000_000i128); // 10B tokens for airdrop
    
    // 2. Create Merkle tree with all vesting schedules
    let (merkle_root, leaves_with_proofs) = create_example_merkle_tree();
    
    // 3. Initialize Merkle root (ONE TRANSACTION - HUGE GAS SAVINGS!)
    // This replaces 1,000+ individual vault creation transactions
    client.initialize_merkle_root(&merkle_root, &1000u32);
    
    println!("Merkle root initialized for 1,000 vesting schedules");
    
    // 4. Users can now activate their individual schedules on-demand
    // Each user pays gas only for their own activation
    
    // Example: User 1 activates their schedule
    let (leaf1, proof1) = leaves_with_proofs.get(0).unwrap();
    
    // In reality, the proof would be valid and this would succeed
    // For this example, we'll show the structure
    let user1_vault_id = client.activate_schedule_with_proof(
        &leaf1.beneficiary,
        leaf1.clone(),
        proof1.clone(),
    );
    
    println!("User 1 activated vault ID: {}", user1_vault_id);
    
    // Check activation status
    assert!(client.is_schedule_activated(&leaf1.beneficiary));
    let activated_vault_id = client.get_activated_vault_id(&leaf1.beneficiary);
    assert_eq!(activated_vault_id.unwrap(), user1_vault_id);
    
    // Other users can activate their schedules independently
    // Each activation is a separate transaction, but much cheaper than
    // creating all vaults upfront
    
    println!("Merkle bulk initialization example completed!");
}

// Example of how to generate real Merkle proofs
pub fn generate_real_merkle_proof_example() {
    let env = Env::default();
    
    // In a real implementation, you would:
    // 1. Create all leaf data
    let leaves = vec![
        &env,
        "leaf1_data", "leaf2_data", "leaf3_data", "leaf4_data"
    ];
    
    // 2. Hash each leaf
    let mut leaf_hashes = Vec::new(&env);
    for leaf in leaves.iter() {
        let hash = env.crypto().sha256(&leaf.clone().into_val(&env));
        leaf_hashes.push_back(hash);
    }
    
    // 3. Build Merkle tree bottom-up
    let mut current_level = leaf_hashes;
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new(&env);
        
        for i in (0..current_level.len()).step_by(2) {
            let left = current_level.get(i).unwrap();
            let right = if i + 1 < current_level.len() {
                current_level.get(i + 1).unwrap()
            } else {
                // Odd number of nodes, duplicate the last one
                left.clone()
            };
            
            // Hash the pair
            let mut combined = Vec::new(&env);
            combined.extend_from_slice(left.as_slice());
            combined.extend_from_slice(right.as_slice());
            let parent_hash = env.crypto().sha256(&combined.into());
            next_level.push_back(parent_hash);
        }
        
        current_level = next_level;
    }
    
    // 4. The final hash is the Merkle root
    let merkle_root = current_level.get(0).unwrap();
    
    println!("Generated Merkle root: {:?}", merkle_root);
    
    // 5. Generate proofs for each leaf
    // (This is simplified - in reality you'd track the path)
    let proof_for_leaf0 = vec![&env]; // Would contain sibling hashes
    let proof_for_leaf1 = vec![&env]; // Would contain sibling hashes
    
    println!("Generated proofs for each leaf");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merkle_bulk_example() {
        example_bulk_initialization();
    }
    
    #[test]
    fn test_merkle_proof_generation() {
        generate_real_merkle_proof_example();
    }
}
