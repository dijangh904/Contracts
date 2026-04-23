#![cfg(test)]
use soroban_sdk::{
    Address,
    Bytes,
    BytesN,
    Env,
    Vec,
    U256,
};
use crate::{
    zk_verifier::{
        ZKVerifier, ZKProof, VerificationKey, AccreditationRecord, 
        AccreditedInvestorInputs, ZKVerifierError,
        ACCREDITED_INVESTOR_CIRCUIT, US_JURISDICTION, EU_JURISDICTION,
        NULLIFIER_MAP, VERIFICATION_KEYS, ACCREDITATION_RECORDS, SUPPORTED_CIRCUITS,
    },
    testutils::{create_test_contract, create_test_address, create_test_env},
};

#[test]
fn test_zk_proof_verification_success() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let investor = create_test_address(&env);
    let admin = create_test_address(&env);

    // Setup verification key
    let verification_key = VerificationKey {
        key_hash: BytesN::from_array([0x01; 32]),
        circuit_type: ACCREDITED_INVESTOR_CIRCUIT,
        supported_jurisdictions: Vec::from_array(&env, [US_JURISDICTION, EU_JURISDICTION]),
        created_at: env.ledger().timestamp(),
        is_active: true,
    };

    // Add verification key
    ZKVerifier::add_verification_key(&env, admin.clone(), verification_key.clone()).unwrap();

    // Add supported circuit
    ZKVerifier::add_supported_circuit(
        &env, 
        admin.clone(), 
        BytesN::from_array([0x01; 32]), 
        ACCREDITED_INVESTOR_CIRCUIT
    ).unwrap();

    // Create valid ZK proof
    let proof = create_valid_zk_proof(&env, US_JURISDICTION);

    // Verify the proof
    let result = ZKVerifier::verify_accredited_investor(&env, proof, investor.clone());
    assert!(result.is_ok());

    // Check that accreditation is recorded
    assert!(ZKVerifier::has_valid_accreditation(&env, investor.clone()));
    
    let record = ZKVerifier::get_accreditation_record(&env, investor).unwrap();
    assert_eq!(record.investor_address, investor);
    assert_eq!(record.jurisdiction_hash, US_JURISDICTION);
}

#[test]
fn test_zk_proof_verification_invalid_nullifier() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let investor = create_test_address(&env);
    let admin = create_test_address(&env);

    // Setup verification key
    let verification_key = VerificationKey {
        key_hash: BytesN::from_array([0x01; 32]),
        circuit_type: ACCREDITED_INVESTOR_CIRCUIT,
        supported_jurisdictions: Vec::from_array(&env, [US_JURISDICTION]),
        created_at: env.ledger().timestamp(),
        is_active: true,
    };

    ZKVerifier::add_verification_key(&env, admin.clone(), verification_key).unwrap();
    ZKVerifier::add_supported_circuit(
        &env, 
        admin.clone(), 
        BytesN::from_array([0x01; 32]), 
        ACCREDITED_INVESTOR_CIRCUIT
    ).unwrap();

    // Create and use first proof
    let proof1 = create_valid_zk_proof(&env, US_JURISDICTION);
    ZKVerifier::verify_accredited_investor(&env, proof1.clone(), investor.clone()).unwrap();

    // Try to use same nullifier again (should fail)
    let investor2 = create_test_address(&env);
    let result = ZKVerifier::verify_accredited_investor(&env, proof1, investor2);
    assert_eq!(result.err(), Some(ZKVerifierError::NullifierAlreadyUsed));
}

#[test]
fn test_zk_proof_verification_unsupported_circuit() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let investor = create_test_address(&env);
    let admin = create_test_address(&env);

    // Setup verification key
    let verification_key = VerificationKey {
        key_hash: BytesN::from_array([0x01; 32]),
        circuit_type: ACCREDITED_INVESTOR_CIRCUIT,
        supported_jurisdictions: Vec::from_array(&env, [US_JURISDICTION]),
        created_at: env.ledger().timestamp(),
        is_active: true,
    };

    ZKVerifier::add_verification_key(&env, admin.clone(), verification_key).unwrap();

    // Create proof with unsupported circuit
    let mut proof = create_valid_zk_proof(&env, US_JURISDICTION);
    proof.circuit_id = BytesN::from_array([0x99; 32]); // Unsupported circuit

    let result = ZKVerifier::verify_accredited_investor(&env, proof, investor);
    assert_eq!(result.err(), Some(ZKVerifierError::UnsupportedCircuit));
}

#[test]
fn test_zk_proof_verification_unsupported_jurisdiction() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let investor = create_test_address(&env);
    let admin = create_test_address(&env);

    // Setup verification key with only US jurisdiction
    let verification_key = VerificationKey {
        key_hash: BytesN::from_array([0x01; 32]),
        circuit_type: ACCREDITED_INVESTOR_CIRCUIT,
        supported_jurisdictions: Vec::from_array(&env, [US_JURISDICTION]), // Only US
        created_at: env.ledger().timestamp(),
        is_active: true,
    };

    ZKVerifier::add_verification_key(&env, admin.clone(), verification_key).unwrap();
    ZKVerifier::add_supported_circuit(
        &env, 
        admin.clone(), 
        BytesN::from_array([0x01; 32]), 
        ACCREDITED_INVESTOR_CIRCUIT
    ).unwrap();

    // Create proof with EU jurisdiction (not supported)
    let proof = create_valid_zk_proof(&env, EU_JURISDICTION);

    let result = ZKVerifier::verify_accredited_investor(&env, proof, investor);
    assert_eq!(result.err(), Some(ZKVerifierError::JurisdictionNotSupported));
}

#[test]
fn test_zk_proof_verification_expired() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let investor = create_test_address(&env);
    let admin = create_test_address(&env);

    // Setup verification key
    let verification_key = VerificationKey {
        key_hash: BytesN::from_array([0x01; 32]),
        circuit_type: ACCREDITED_INVESTOR_CIRCUIT,
        supported_jurisdictions: Vec::from_array(&env, [US_JURISDICTION]),
        created_at: env.ledger().timestamp(),
        is_active: true,
    };

    ZKVerifier::add_verification_key(&env, admin.clone(), verification_key).unwrap();
    ZKVerifier::add_supported_circuit(
        &env, 
        admin.clone(), 
        BytesN::from_array([0x01; 32]), 
        ACCREDITED_INVESTOR_CIRCUIT
    ).unwrap();

    // Create expired proof
    let mut proof = create_valid_zk_proof(&env, US_JURISDICTION);
    
    // Modify public inputs to set expiry in the past
    let mut public_inputs = Vec::new(&env);
    
    // jurisdiction_hash
    public_inputs.push_back(Bytes::from_array(&env, &US_JURISDICTION.to_array()));
    // net_worth_threshold_met (true)
    public_inputs.push_back(Bytes::from_array(&env, &[1]));
    // income_threshold_met (true)
    public_inputs.push_back(Bytes::from_array(&env, &[1]));
    // professional_certifications (true)
    public_inputs.push_back(Bytes::from_array(&env, &[1]));
    // timestamp (current time)
    let timestamp = env.ledger().timestamp().to_be_bytes();
    public_inputs.push_back(Bytes::from_array(&env, &timestamp));
    // expiry (past time)
    let past_expiry = (env.ledger().timestamp() - 1000).to_be_bytes();
    public_inputs.push_back(Bytes::from_array(&env, &past_expiry));
    
    proof.public_inputs = public_inputs;

    let result = ZKVerifier::verify_accredited_investor(&env, proof, investor);
    assert_eq!(result.err(), Some(ZKVerifierError::AccreditationExpired));
}

#[test]
fn test_accreditation_record_expiry() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let investor = create_test_address(&env);
    let admin = create_test_address(&env);

    // Setup verification key
    let verification_key = VerificationKey {
        key_hash: BytesN::from_array([0x01; 32]),
        circuit_type: ACCREDITED_INVESTOR_CIRCUIT,
        supported_jurisdictions: Vec::from_array(&env, [US_JURISDICTION]),
        created_at: env.ledger().timestamp(),
        is_active: true,
    };

    ZKVerifier::add_verification_key(&env, admin.clone(), verification_key).unwrap();
    ZKVerifier::add_supported_circuit(
        &env, 
        admin.clone(), 
        BytesN::from_array([0x01; 32]), 
        ACCREDITED_INVESTOR_CIRCUIT
    ).unwrap();

    // Create proof with short expiry
    let mut proof = create_valid_zk_proof(&env, US_JURISDICTION);
    
    // Set expiry to 1 second from now
    let mut public_inputs = Vec::new(&env);
    public_inputs.push_back(Bytes::from_array(&env, &US_JURISDICTION.to_array()));
    public_inputs.push_back(Bytes::from_array(&env, &[1]));
    public_inputs.push_back(Bytes::from_array(&env, &[1]));
    public_inputs.push_back(Bytes::from_array(&env, &[1]));
    let timestamp = env.ledger().timestamp().to_be_bytes();
    public_inputs.push_back(Bytes::from_array(&env, &timestamp));
    let soon_expiry = (env.ledger().timestamp() + 1).to_be_bytes();
    public_inputs.push_back(Bytes::from_array(&env, &soon_expiry));
    
    proof.public_inputs = public_inputs;

    // Verify the proof
    ZKVerifier::verify_accredited_investor(&env, proof, investor.clone()).unwrap();
    
    // Should be valid initially
    assert!(ZKVerifier::has_valid_accreditation(&env, investor.clone()));
    
    // Advance time past expiry
    env.ledger().set_timestamp(env.ledger().timestamp() + 2);
    
    // Should no longer be valid
    assert!(!ZKVerifier::has_valid_accreditation(&env, investor));
}

#[test]
fn test_verification_key_management() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let admin = create_test_address(&env);

    // Add verification key
    let verification_key = VerificationKey {
        key_hash: BytesN::from_array([0x01; 32]),
        circuit_type: ACCREDITED_INVESTOR_CIRCUIT,
        supported_jurisdictions: Vec::from_array(&env, [US_JURISDICTION]),
        created_at: env.ledger().timestamp(),
        is_active: true,
    };

    let result = ZKVerifier::add_verification_key(&env, admin.clone(), verification_key.clone());
    assert!(result.is_ok());

    // Retrieve and verify the key
    let retrieved_key = ZKVerifier::get_verification_key(&env, verification_key.key_hash).unwrap();
    assert_eq!(retrieved_key.key_hash, verification_key.key_hash);
    assert_eq!(retrieved_key.circuit_type, verification_key.circuit_type);
    assert!(retrieved_key.is_active);
}

#[test]
fn test_inactive_verification_key() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let investor = create_test_address(&env);
    let admin = create_test_address(&env);

    // Add inactive verification key
    let verification_key = VerificationKey {
        key_hash: BytesN::from_array([0x01; 32]),
        circuit_type: ACCREDITED_INVESTOR_CIRCUIT,
        supported_jurisdictions: Vec::from_array(&env, [US_JURISDICTION]),
        created_at: env.ledger().timestamp(),
        is_active: false, // Inactive
    };

    ZKVerifier::add_verification_key(&env, admin.clone(), verification_key).unwrap();
    ZKVerifier::add_supported_circuit(
        &env, 
        admin.clone(), 
        BytesN::from_array([0x01; 32]), 
        ACCREDITED_INVESTOR_CIRCUIT
    ).unwrap();

    // Create proof with inactive key
    let proof = create_valid_zk_proof(&env, US_JURISDICTION);

    let result = ZKVerifier::verify_accredited_investor(&env, proof, investor);
    assert_eq!(result.err(), Some(ZKVerifierError::InvalidVerificationKey));
}

// Helper function to create a valid ZK proof for testing
fn create_valid_zk_proof(env: &Env, jurisdiction: BytesN<32>) -> ZKProof {
    let mut public_inputs = Vec::new(env);
    
    // jurisdiction_hash
    public_inputs.push_back(Bytes::from_array(env, &jurisdiction.to_array()));
    // net_worth_threshold_met (true)
    public_inputs.push_back(Bytes::from_array(env, &[1]));
    // income_threshold_met (true)
    public_inputs.push_back(Bytes::from_array(env, &[1]));
    // professional_certifications (true)
    public_inputs.push_back(Bytes::from_array(env, &[1]));
    // timestamp (current time)
    let timestamp = env.ledger().timestamp().to_be_bytes();
    public_inputs.push_back(Bytes::from_array(env, &timestamp));
    // expiry (future time)
    let future_expiry = (env.ledger().timestamp() + 86400).to_be_bytes(); // 24 hours from now
    public_inputs.push_back(Bytes::from_array(env, &future_expiry));

    ZKProof {
        proof_data: Bytes::from_array(env, &[0x01, 0x02, 0x03]), // Mock proof data
        public_inputs,
        nullifier: BytesN::from_array([0x42; 32]), // Unique nullifier
        circuit_id: BytesN::from_array([0x01; 32]), // Supported circuit
        verification_key_hash: BytesN::from_array([0x01; 32]),
    }
}

#[test]
fn test_accredited_investor_vault_creation() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let investor = create_test_address(&env);
    let admin = create_test_address(&env);
    let token_admin = create_test_address(&env);

    // Setup token and contract
    let token_address = create_test_address(&env);
    
    // Setup verification key
    let verification_key = VerificationKey {
        key_hash: BytesN::from_array([0x01; 32]),
        circuit_type: ACCREDITED_INVESTOR_CIRCUIT,
        supported_jurisdictions: Vec::from_array(&env, [US_JURISDICTION]),
        created_at: env.ledger().timestamp(),
        is_active: true,
    };

    ZKVerifier::add_verification_key(&env, admin.clone(), verification_key).unwrap();
    ZKVerifier::add_supported_circuit(
        &env, 
        admin.clone(), 
        BytesN::from_array([0x01; 32]), 
        ACCREDITED_INVESTOR_CIRCUIT
    ).unwrap();

    // First verify the investor is accredited
    let proof = create_valid_zk_proof(&env, US_JURISDICTION);
    ZKVerifier::verify_accredited_investor(&env, proof, investor.clone()).unwrap();

    // Now create accredited-only vault (should succeed)
    let vault_id = crate::VestingContract::create_vault_accredited_only(
        env.clone(),
        investor.clone(),
        1000,
        token_address,
        env.ledger().timestamp(),
        env.ledger().timestamp() + 1000,
        10,
        true,
        true,
        1,
    );

    assert!(vault_id > 0);

    // Try with non-accredited investor (should fail)
    let non_accredited = create_test_address(&env);
    let result = std::panic::catch_unwind(|| {
        crate::VestingContract::create_vault_accredited_only(
            env.clone(),
            non_accredited,
            1000,
            token_address,
            env.ledger().timestamp(),
            env.ledger().timestamp() + 1000,
            10,
            true,
            true,
            1,
        )
    });

    assert!(result.is_err());
}
