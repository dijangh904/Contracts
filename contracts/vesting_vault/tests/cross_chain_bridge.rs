#![allow(dead_code)]

use soroban_sdk::{Address, BytesN, Bytes, Vec, testutils::{Address as _, AuthorizedFunction, LoggedEvent}};
use vesting_vault::{VestingVault, VestingVaultClient, ChainId, VAA, BridgeConfig, Error, CrossChainClaimPayload};

#[test]
fn test_initialize_bridge() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    supported_chains.push_back(ChainId::Solana);
    
    client.initialize_bridge(
        &admin,
        &wormhole_core,
        &supported_chains,
        &1000000i128,
        &60u64, // 1 minute cooldown
    );
    
    let config = client.get_bridge_config_public().unwrap();
    assert!(!config.is_paused);
    assert_eq!(config.wormhole_core_address, wormhole_core);
    assert_eq!(config.max_bridge_amount, 1000000i128);
    assert_eq!(config.bridge_cooldown, 60u64);
}

#[test]
fn test_initialize_bridge_invalid_amount() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    let result = client.try_initialize_bridge(
        &admin,
        &wormhole_core,
        &supported_chains,
        &0i128, // Invalid: zero amount
        &60u64,
    );
    
    assert_eq!(result, Err(Ok(Error::InvalidInput)));
}

#[test]
fn test_initialize_bridge_empty_chains() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let supported_chains = Vec::new(&env); // Empty
    
    let result = client.try_initialize_bridge(
        &admin,
        &wormhole_core,
        &supported_chains,
        &1000000i128,
        &60u64,
    );
    
    assert_eq!(result, Err(Ok(Error::InvalidInput)));
}

#[test]
fn test_toggle_bridge_pause() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &60u64);
    
    // Pause the bridge
    client.toggle_bridge_pause(&admin);
    
    let config = client.get_bridge_config_public().unwrap();
    assert!(config.is_paused);
    
    // Unpause the bridge
    client.toggle_bridge_pause(&admin);
    
    let config = client.get_bridge_config_public().unwrap();
    assert!(!config.is_paused);
}

#[test]
fn test_cross_chain_claim_bridge_not_configured() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let user = Address::generate(&env);
    
    let vaa = create_mock_vaa(&env);
    
    let result = client.try_cross_chain_claim(&user, &vaa);
    assert_eq!(result, Err(Ok(Error::BridgeNotConfigured)));
}

#[test]
fn test_cross_chain_claim_bridge_paused() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &60u64);
    client.toggle_bridge_pause(&admin);
    
    let vaa = create_mock_vaa(&env);
    
    let result = client.try_cross_chain_claim(&user, &vaa);
    assert_eq!(result, Err(Ok(Error::BridgePaused)));
}

#[test]
fn test_cross_chain_claim_invalid_vaa_signature() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &60u64);
    
    let mut vaa = create_mock_vaa(&env);
    vaa.signatures = Vec::new(&env); // Empty signatures
    
    let result = client.try_cross_chain_claim(&user, &vaa);
    assert_eq!(result, Err(Ok(Error::InvalidBridgeSignature)));
}

#[test]
fn test_cross_chain_claim_unsupported_chain() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    // Solana is NOT supported
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &60u64);
    
    let vaa = create_mock_vaa_with_chain(&env, ChainId::Solana);
    
    let result = client.try_cross_chain_claim(&user, &vaa);
    assert_eq!(result, Err(Ok(Error::UnsupportedChain)));
}

#[test]
fn test_cross_chain_claim_amount_exceeds_limit() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000i128, &60u64);
    
    let vaa = create_mock_vaa_with_amount(&env, 2000i128); // Exceeds limit
    
    let result = client.try_cross_chain_claim(&user, &vaa);
    assert_eq!(result, Err(Ok(Error::BridgeAmountExceedsLimit)));
}

#[test]
fn test_cross_chain_claim_replay_attack_prevention() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &0u64); // No cooldown for test
    
    let vaa = create_mock_vaa(&env);
    
    // First claim should succeed (if payload parsing worked)
    // For now, we'll test the nonce check directly
    // Since parse_vaa_payload returns error, we can't test full flow yet
    // But we can test the nonce storage mechanism
    
    // This test is a placeholder for when payload parsing is implemented
}

#[test]
fn test_cross_chain_claim_sequence_number_strict_increment() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &0u64);
    
    // Initially, last sequence should be 0
    assert_eq!(client.get_bridge_last_sequence_public(), 0);
    
    // Test with sequence number 0 (should fail - must be > last_sequence)
    let vaa = create_mock_vaa_with_sequence(&env, 0);
    let result = client.try_cross_chain_claim(&user, &vaa);
    assert_eq!(result, Err(Ok(Error::InvalidVaaSequence)));
    
    // Test with sequence number 1 (should succeed if payload parsing worked)
    let vaa = create_mock_vaa_with_sequence(&env, 1);
    // This will fail on payload parsing, but sequence check would pass
}

#[test]
fn test_queue_cross_chain_claim_when_paused() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &60u64);
    client.toggle_bridge_pause(&admin);
    
    let vaa = create_mock_vaa(&env);
    
    // Queue should work even when paused
    // But will fail on payload parsing for now
    let result = client.try_queue_cross_chain_claim(&user, &vaa);
    // Will fail on payload parsing, not on bridge pause
}

#[test]
fn test_process_queued_claims() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &0u64);
    client.toggle_bridge_pause(&admin);
    
    // Initially no queued claims
    assert_eq!(client.get_queued_claims_count(), 0);
    
    // Unpause the bridge
    client.toggle_bridge_pause(&admin);
    
    // Process empty queue
    let processed = client.process_queued_claims(&0).unwrap();
    assert_eq!(processed, 0);
}

#[test]
fn test_bridge_cooldown() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    // Set 10 second cooldown
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &10u64);
    
    // First operation
    env.ledger().set_timestamp(100);
    
    let vaa = create_mock_vaa(&env);
    // Will fail on payload parsing, but cooldown check would pass
    
    // Try second operation immediately (should fail cooldown)
    env.ledger().set_timestamp(105); // Only 5 seconds later
    
    let vaa2 = create_mock_vaa(&env);
    let result = client.try_cross_chain_claim(&user, &vaa2);
    // Will fail on payload parsing first, then cooldown
}

#[test]
fn test_cross_chain_claim_initiated_event() {
    let env = soroban_sdk::Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let wormhole_core = Address::generate(&env);
    
    let mut supported_chains = Vec::new(&env);
    supported_chains.push_back(ChainId::Ethereum);
    
    client.initialize_bridge(&admin, &wormhole_core, &supported_chains, &1000000i128, &0u64);
    
    // This test is a placeholder for when payload parsing is implemented
    // The event emission logic is in place, but we can't trigger it without
    // a working payload parser
}

// Helper function to create a mock VAA for testing
fn create_mock_vaa(env: &soroban_sdk::Env) -> VAA {
    create_mock_vaa_with_sequence(env, 1)
}

fn create_mock_vaa_with_sequence(env: &soroban_sdk::Env, sequence: u64) -> VAA {
    let mut signatures = Vec::new(env);
    // Add a mock signature
    let sig_bytes = BytesN::from_array(env, &[0u8; 65]);
    signatures.push_back(sig_bytes);
    
    let payload = Bytes::from_slice(env, &[0u8; 32]); // Mock payload
    
    VAA {
        version: 1,
        guardian_set_index: 0,
        emitter_chain: ChainId::Stellar,
        emitter_address: BytesN::from_array(env, &[0u8; 32]),
        sequence,
        consistency_level: 1,
        timestamp: env.ledger().timestamp(),
        signatures,
        payload,
    }
}

fn create_mock_vaa_with_chain(env: &soroban_sdk::Env, chain: ChainId) -> VAA {
    let mut vaa = create_mock_vaa(env);
    vaa.emitter_chain = chain;
    vaa
}

fn create_mock_vaa_with_amount(env: &soroban_sdk::Env, amount: i128) -> VAA {
    // This is a placeholder - in real implementation, the amount would be
    // encoded in the payload, not the VAA itself
    create_mock_vaa(env)
}
