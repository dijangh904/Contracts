#![cfg(test)]
use soroban_sdk::{
    Address,
    Bytes,
    BytesN,
    Env,
    Vec,
};
use crate::{
    legal_saft::{
        LegalSAFTManager, LegalDocument, DocumentSignature, VaultLegalDocuments,
        DocumentType, LegalSAFTError,
        LEGAL_DOCUMENTS, DOCUMENT_SIGNATURES, VAULT_LEGAL_DOCS,
    },
    testutils::{create_test_contract, create_test_address, create_test_env},
};

#[test]
fn test_store_legal_document_success() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let admin = create_test_address(&env);
    let vault_id = 1u64;

    let ipfs_cid = "QmTest123456789".to_string();
    let document_hash = BytesN::from_array([0x01; 32]);
    let jurisdiction = "United States".to_string();
    let version = "1.0".to_string();

    let result = LegalSAFTManager::store_legal_hash(
        &env,
        admin.clone(),
        vault_id,
        DocumentType::SAFT,
        ipfs_cid.clone(),
        document_hash.clone(),
        jurisdiction.clone(),
        version.clone(),
        None,
    );

    assert!(result.is_ok());

    // Verify document was stored
    let stored_doc = LegalSAFTManager::get_legal_document(&env, document_hash.clone()).unwrap();
    assert_eq!(stored_doc.document_type, DocumentType::SAFT);
    assert_eq!(stored_doc.ipfs_cid, ipfs_cid);
    assert_eq!(stored_doc.document_hash, document_hash);
    assert_eq!(stored_doc.admin_address, admin);
    assert_eq!(stored_doc.jurisdiction, jurisdiction);
    assert_eq!(stored_doc.version, version);
}

#[test]
fn test_store_legal_document_invalid_ipfs() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let admin = create_test_address(&env);
    let vault_id = 1u64;

    let invalid_ipfs_cid = "invalid_cid".to_string();
    let document_hash = BytesN::from_array([0x01; 32]);

    let result = LegalSAFTManager::store_legal_hash(
        &env,
        admin,
        vault_id,
        DocumentType::SAFT,
        invalid_ipfs_cid,
        document_hash,
        "US".to_string(),
        "1.0".to_string(),
        None,
    );

    assert_eq!(result.err(), Some(LegalSAFTError::InvalidIPFSCid));
}

#[test]
fn test_sign_legal_document_success() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let admin = create_test_address(&env);
    let beneficiary = create_test_address(&env);
    let vault_id = 1u64;

    // First store a legal document
    let document_hash = BytesN::from_array([0x01; 32]);
    LegalSAFTManager::store_legal_hash(
        &env,
        admin,
        vault_id,
        DocumentType::SAFT,
        "QmTest123".to_string(),
        document_hash.clone(),
        "US".to_string(),
        "1.0".to_string(),
        None,
    ).unwrap();

    // Sign the document
    let signature = Bytes::from_array(&[0x01, 0x02, 0x03]);
    let message = "I agree to terms".to_string();

    let result = LegalSAFTManager::sign_legal_document(
        &env,
        beneficiary.clone(),
        vault_id,
        document_hash.clone(),
        signature.clone(),
        message.clone(),
    );

    assert!(result.is_ok());

    // Verify signature was stored
    let stored_sig = LegalSAFTManager::get_document_signature(
        &env,
        beneficiary.clone(),
        document_hash.clone()
    ).unwrap();
    assert_eq!(stored_sig.beneficiary, beneficiary);
    assert_eq!(stored_sig.document_hash, document_hash);
    assert_eq!(stored_sig.signature, signature);
    assert_eq!(stored_sig.message, message);
}

#[test]
fn test_sign_legal_document_already_signed() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let admin = create_test_address(&env);
    let beneficiary = create_test_address(&env);
    let vault_id = 1u64;

    let document_hash = BytesN::from_array([0x01; 32]);
    LegalSAFTManager::store_legal_hash(
        &env,
        admin,
        vault_id,
        DocumentType::SAFT,
        "QmTest123".to_string(),
        document_hash.clone(),
        "US".to_string(),
        "1.0".to_string(),
        None,
    ).unwrap();

    // Sign once
    LegalSAFTManager::sign_legal_document(
        &env,
        beneficiary.clone(),
        vault_id,
        document_hash.clone(),
        Bytes::from_array(&[0x01]),
        "First signature".to_string(),
    ).unwrap();

    // Try to sign again
    let result = LegalSAFTManager::sign_legal_document(
        &env,
        beneficiary,
        vault_id,
        document_hash,
        Bytes::from_array(&[0x02]),
        "Second signature".to_string(),
    );

    assert_eq!(result.err(), Some(LegalSAFTError::SignatureAlreadyExists));
}

#[test]
fn test_all_documents_signed_check() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let admin = create_test_address(&env);
    let beneficiary = create_test_address(&env);
    let vault_id = 1u64;

    // Store two legal documents
    let doc1_hash = BytesN::from_array([0x01; 32]);
    let doc2_hash = BytesN::from_array([0x02; 32]);

    LegalSAFTManager::store_legal_hash(
        &env,
        admin.clone(),
        vault_id,
        DocumentType::SAFT,
        "QmTest1".to_string(),
        doc1_hash.clone(),
        "US".to_string(),
        "1.0".to_string(),
        None,
    ).unwrap();

    LegalSAFTManager::store_legal_hash(
        &env,
        admin,
        vault_id,
        DocumentType::GrantAgreement,
        "QmTest2".to_string(),
        doc2_hash.clone(),
        "US".to_string(),
        "1.0".to_string(),
        None,
    ).unwrap();

    // Initially not all signed
    assert!(!LegalSAFTManager::are_all_documents_signed(&env, vault_id));

    // Sign first document
    LegalSAFTManager::sign_legal_document(
        &env,
        beneficiary.clone(),
        vault_id,
        doc1_hash.clone(),
        Bytes::from_array(&[0x01]),
        "Signed SAFT".to_string(),
    ).unwrap();

    // Still not all signed
    assert!(!LegalSAFTManager::are_all_documents_signed(&env, vault_id));

    // Sign second document
    LegalSAFTManager::sign_legal_document(
        &env,
        beneficiary,
        vault_id,
        doc2_hash,
        Bytes::from_array(&[0x02]),
        "Signed Grant".to_string(),
    ).unwrap();

    // Now all should be signed
    assert!(LegalSAFTManager::are_all_documents_signed(&env, vault_id));
}

#[test]
fn test_vault_legal_documents_status() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let vault_id = 1u64;

    // Initially no legal documents
    let status = LegalSAFTManager::get_vault_legal_documents(&env, vault_id);
    assert!(status.is_none());

    // Add document requirement
    let doc_hash = BytesN::from_array([0x01; 32]);
    LegalSAFTManager::store_legal_hash(
        &env,
        create_test_address(&env),
        vault_id,
        DocumentType::SAFT,
        "QmTest123".to_string(),
        doc_hash,
        "US".to_string(),
        "1.0".to_string(),
        None,
    ).unwrap();

    // Check status
    let status = LegalSAFTManager::get_vault_legal_documents(&env, vault_id).unwrap();
    assert_eq!(status.vault_id, vault_id);
    assert_eq!(status.required_documents.len(), 1);
    assert_eq!(status.signed_documents.len(), 0);
    assert!(!status.all_documents_signed);
    assert!(!status.vesting_can_start);
}

#[test]
fn test_revoke_legal_document() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let admin = create_test_address(&env);
    let vault_id = 1u64;

    let document_hash = BytesN::from_array([0x01; 32]);
    LegalSAFTManager::store_legal_hash(
        &env,
        admin.clone(),
        vault_id,
        DocumentType::SAFT,
        "QmTest123".to_string(),
        document_hash,
        "US".to_string(),
        "1.0".to_string(),
        None,
    ).unwrap();

    // Revoke document
    let reason = "Document updated".to_string();
    let result = LegalSAFTManager::revoke_legal_document(
        &env,
        admin.clone(),
        document_hash,
        reason.clone(),
    );

    assert!(result.is_ok());

    // Verify document is expired
    let doc = LegalSAFTManager::get_legal_document(&env, document_hash).unwrap();
    assert!(doc.expires_at.is_some());
    assert!(doc.expires_at.unwrap() > 0);
}

#[test]
fn test_vault_with_legal_requirements() {
    let env = create_test_env();
    let contract_id = create_test_contract(&env);
    let owner = create_test_address(&env);
    let token_address = create_test_address(&env);

    // Create vault with legal requirements
    let vault_id = crate::VestingContract::create_vault_with_legal_requirements(
        env.clone(),
        owner.clone(),
        1000,
        token_address,
        env.ledger().timestamp(),
        env.ledger().timestamp() + 1000,
        10,
        true,
        true,
        1,
        true, // requires legal signatures
    );

    // Check vault has legal requirements
    let vault = crate::VestingContract::get_vault_internal(&env, vault_id);
    assert!(vault.requires_legal_signatures);
    assert!(!vault.legal_documents_signed); // Initially not signed

    // Should not be able to claim without signatures
    let result = std::panic::catch_unwind(|| {
        crate::VestingContract::claim_tokens(env.clone(), vault_id, 100);
    });
    assert!(result.is_err());
}
