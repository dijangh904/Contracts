#![no_std]
use soroban_sdk::{
    contracttype,
    contracterror,
    Address,
    Env,
    Vec,
    Bytes,
    BytesN,
    Symbol,
    String,
};

/// Legal document anchoring errors
#[contracterror]
#[repr(u32)]
pub enum LegalSAFTError {
    InvalidIPFSCid = 1,
    DocumentAlreadyAnchored = 2,
    DocumentNotAnchored = 3,
    InvalidSignature = 4,
    SignatureAlreadyExists = 5,
    UnauthorizedDocumentAccess = 6,
    DocumentHashMismatch = 7,
    VaultNotInitialized = 8,
    BeneficiarySignatureRequired = 9,
    AdminSignatureRequired = 10,
    InvalidDocumentType = 11,
    DocumentExpired = 12,
}

/// Document types for legal agreements
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentType {
    SAFT,           // Simple Agreement for Future Tokens
    GrantAgreement,  // Grant Agreement
    PurchaseAgreement, // Purchase Agreement
    TokenWarrant,    // Token Warrant
    ConvertibleNote,  // Convertible Note
}

/// Legal document metadata stored on-chain
#[derive(Clone)]
#[contracttype]
pub struct LegalDocument {
    pub document_type: DocumentType,
    pub ipfs_cid: String,           // IPFS CID of the document
    pub document_hash: BytesN<32>,   // SHA-256 hash of the document
    pub admin_address: Address,       // Admin who anchored the document
    pub anchored_at: u64,            // Timestamp when document was anchored
    pub expires_at: Option<u64>,     // Optional expiry timestamp
    pub jurisdiction: String,         // Legal jurisdiction
    pub version: String,             // Document version
}

/// Beneficiary signature for legal document
#[derive(Clone)]
#[contracttype]
pub struct DocumentSignature {
    pub beneficiary: Address,        // Beneficiary who signed
    pub document_hash: BytesN<32>,  // Hash of the document being signed
    pub signature: Bytes,            // Cryptographic signature
    pub signed_at: u64,             // Timestamp when signed
    pub message: String,             // Optional message with signature
}

/// Vault legal document association
#[derive(Clone)]
#[contracttype]
pub struct VaultLegalDocuments {
    pub vault_id: u64,
    pub required_documents: Vec<BytesN<32>>, // List of required document hashes
    pub signed_documents: Vec<BytesN<32>>,   // List of signed document hashes
    pub all_documents_signed: bool,            // Whether all required documents are signed
    pub vesting_can_start: bool,               // Whether vesting can start
}

/// Storage keys for legal SAFT system
pub const LEGAL_DOCUMENTS: Bytes = Bytes::from_short_bytes("LEGAL_DOCUMENTS");
pub const DOCUMENT_SIGNATURES: Bytes = Bytes::from_short_bytes("DOCUMENT_SIGNATURES");
pub const VAULT_LEGAL_DOCS: Bytes = Bytes::from_short_bytes("VAULT_LEGAL_DOCS");
pub const DOCUMENT_INDEX: Bytes = Bytes::from_short_bytes("DOCUMENT_INDEX");

/// Legal SAFT Manager implementation
pub struct LegalSAFTManager;

impl LegalSAFTManager {
    /// Store legal document hash anchored by admin
    pub fn store_legal_hash(
        env: &Env,
        admin: Address,
        vault_id: u64,
        document_type: DocumentType,
        ipfs_cid: String,
        document_hash: BytesN<32>,
        jurisdiction: String,
        version: String,
        expires_at: Option<u64>,
    ) -> Result<(), LegalSAFTError> {
        // Validate IPFS CID format (basic validation)
        if ipfs_cid.len() < 10 || !ipfs_cid.starts_with("Qm") {
            return Err(LegalSAFTError::InvalidIPFSCid);
        }

        // Check if document already anchored
        let doc_key = (LEGAL_DOCUMENTS, document_hash.clone());
        if env.storage().persistent().has(&doc_key) {
            return Err(LegalSAFTError::DocumentAlreadyAnchored);
        }

        // Create legal document record
        let document = LegalDocument {
            document_type: document_type.clone(),
            ipfs_cid: ipfs_cid.clone(),
            document_hash: document_hash.clone(),
            admin_address: admin.clone(),
            anchored_at: env.ledger().timestamp(),
            expires_at,
            jurisdiction: jurisdiction.clone(),
            version: version.clone(),
        };

        // Store document
        env.storage().persistent().set(&doc_key, &document);

        // Update vault legal documents
        Self::add_document_to_vault(env, vault_id, document_hash.clone())?;

        // Emit event
        LegalDocumentAnchored {
            vault_id,
            document_type,
            ipfs_cid,
            document_hash,
            admin,
            jurisdiction,
            version,
        }.publish(env);

        Ok(())
    }

    /// Beneficiary signs a legal document hash
    pub fn sign_legal_document(
        env: &Env,
        beneficiary: Address,
        vault_id: u64,
        document_hash: BytesN<32>,
        signature: Bytes,
        message: String,
    ) -> Result<(), LegalSAFTError> {
        // Verify document exists
        let doc_key = (LEGAL_DOCUMENTS, document_hash.clone());
        let document: LegalDocument = env.storage().persistent()
            .get(&doc_key)
            .ok_or(LegalSAFTError::DocumentNotAnchored)?;

        // Check if document has expired
        if let Some(expiry) = document.expires_at {
            if env.ledger().timestamp() > expiry {
                return Err(LegalSAFTError::DocumentExpired);
            }
        }

        // Check if signature already exists
        let sig_key = (DOCUMENT_SIGNATURES, beneficiary.clone(), document_hash.clone());
        if env.storage().persistent().has(&sig_key) {
            return Err(LegalSAFTError::SignatureAlreadyExists);
        }

        // Create signature record
        let doc_signature = DocumentSignature {
            beneficiary: beneficiary.clone(),
            document_hash: document_hash.clone(),
            signature: signature.clone(),
            signed_at: env.ledger().timestamp(),
            message: message.clone(),
        };

        // Store signature
        env.storage().persistent().set(&sig_key, &doc_signature);

        // Update vault legal documents
        Self::mark_document_signed(env, vault_id, document_hash.clone())?;

        // Emit event
        DocumentSigned {
            vault_id,
            beneficiary,
            document_hash,
            signature,
            message,
        }.publish(env);

        Ok(())
    }

    /// Check if all required documents are signed for a vault
    pub fn are_all_documents_signed(env: &Env, vault_id: u64) -> bool {
        let vault_key = (VAULT_LEGAL_DOCS, vault_id);
        if let Some(vault_docs) = env.storage().persistent().get::<_, VaultLegalDocuments>(&vault_key) {
            vault_docs.all_documents_signed && vault_docs.vesting_can_start
        } else {
            false // No legal documents required
        }
    }

    /// Get legal document by hash
    pub fn get_legal_document(env: &Env, document_hash: BytesN<32>) -> Option<LegalDocument> {
        let doc_key = (LEGAL_DOCUMENTS, document_hash);
        env.storage().persistent().get(&doc_key)
    }

    /// Get document signature
    pub fn get_document_signature(
        env: &Env,
        beneficiary: Address,
        document_hash: BytesN<32>
    ) -> Option<DocumentSignature> {
        let sig_key = (DOCUMENT_SIGNATURES, beneficiary, document_hash);
        env.storage().persistent().get(&sig_key)
    }

    /// Get vault legal documents status
    pub fn get_vault_legal_documents(env: &Env, vault_id: u64) -> Option<VaultLegalDocuments> {
        let vault_key = (VAULT_LEGAL_DOCS, vault_id);
        env.storage().persistent().get(&vault_key)
    }

    /// Get all documents for a vault
    pub fn get_vault_documents(env: &Env, vault_id: u64) -> Vec<LegalDocument> {
        let vault_key = (VAULT_LEGAL_DOCS, vault_id);
        if let Some(vault_docs) = env.storage().persistent().get::<_, VaultLegalDocuments>(&vault_key) {
            let mut documents = Vec::new(env);
            
            for doc_hash in vault_docs.required_documents.iter() {
                if let Some(document) = Self::get_legal_document(env, doc_hash) {
                    documents.push_back(document);
                }
            }
            
            documents
        } else {
            Vec::new(env)
        }
    }

    /// Get signed documents for a beneficiary
    pub fn get_beneficiary_signed_documents(
        env: &Env,
        beneficiary: Address
    ) -> Vec<DocumentSignature> {
        // This is a simplified implementation
        // In production, you might want to maintain an index
        Vec::new(env)
    }

    /// Revoke a legal document (admin only)
    pub fn revoke_legal_document(
        env: &Env,
        admin: Address,
        document_hash: BytesN<32>,
        reason: String,
    ) -> Result<(), LegalSAFTError> {
        let doc_key = (LEGAL_DOCUMENTS, document_hash.clone());
        let mut document: LegalDocument = env.storage().persistent()
            .get(&doc_key)
            .ok_or(LegalSAFTError::DocumentNotAnchored)?;

        // Verify admin authorization (in production, add proper admin check)
        if document.admin_address != admin {
            return Err(LegalSAFTError::UnauthorizedDocumentAccess);
        }

        // Mark document as expired (soft delete)
        document.expires_at = Some(env.ledger().timestamp());
        env.storage().persistent().set(&doc_key, &document);

        // Emit event
        LegalDocumentRevoked {
            document_hash,
            admin,
            reason,
        }.publish(env);

        Ok(())
    }

    // Private helper methods

    fn add_document_to_vault(
        env: &Env,
        vault_id: u64,
        document_hash: BytesN<32>,
    ) -> Result<(), LegalSAFTError> {
        let vault_key = (VAULT_LEGAL_DOCS, vault_id);
        
        let mut vault_docs = if let Some(existing) = env.storage().persistent()
            .get::<_, VaultLegalDocuments>(&vault_key) {
            existing
        } else {
            VaultLegalDocuments {
                vault_id,
                required_documents: Vec::new(env),
                signed_documents: Vec::new(env),
                all_documents_signed: false,
                vesting_can_start: false,
            }
        };

        // Add to required documents if not already present
        if !vault_docs.required_documents.iter().any(|h| h == &document_hash) {
            vault_docs.required_documents.push_back(document_hash.clone());
        }

        // Update status
        vault_docs.all_documents_signed = false;
        vault_docs.vesting_can_start = false;

        env.storage().persistent().set(&vault_key, &vault_docs);
        Ok(())
    }

    fn mark_document_signed(
        env: &Env,
        vault_id: u64,
        document_hash: BytesN<32>,
    ) -> Result<(), LegalSAFTError> {
        let vault_key = (VAULT_LEGAL_DOCS, vault_id);
        
        let mut vault_docs = env.storage().persistent()
            .get::<_, VaultLegalDocuments>(&vault_key)
            .ok_or(LegalSAFTError::VaultNotInitialized)?;

        // Add to signed documents if not already present
        if !vault_docs.signed_documents.iter().any(|h| h == &document_hash) {
            vault_docs.signed_documents.push_back(document_hash.clone());
        }

        // Check if all required documents are signed
        let all_signed = vault_docs.required_documents.iter().all(|req_hash| {
            vault_docs.signed_documents.iter().any(|sig_hash| sig_hash == req_hash)
        });

        vault_docs.all_documents_signed = all_signed;
        vault_docs.vesting_can_start = all_signed;

        env.storage().persistent().set(&vault_key, &vault_docs);

        // Emit event if all documents are signed
        if all_signed {
            AllDocumentsSigned {
                vault_id,
                beneficiary: env.current_contract_address(), // This would be the vault owner
            }.publish(env);
        }

        Ok(())
    }
}

/// Events for legal document operations
#[contractevent]
pub struct LegalDocumentAnchored {
    #[topic]
    pub vault_id: u64,
    #[topic]
    pub document_type: DocumentType,
    #[topic]
    pub ipfs_cid: String,
    #[topic]
    pub document_hash: BytesN<32>,
    pub admin: Address,
    pub jurisdiction: String,
    pub version: String,
}

#[contractevent]
pub struct DocumentSigned {
    #[topic]
    pub vault_id: u64,
    #[topic]
    pub beneficiary: Address,
    #[topic]
    pub document_hash: BytesN<32>,
    pub signature: Bytes,
    pub message: String,
}

#[contractevent]
pub struct AllDocumentsSigned {
    #[topic]
    pub vault_id: u64,
    #[topic]
    pub beneficiary: Address,
}

#[contractevent]
pub struct LegalDocumentRevoked {
    #[topic]
    pub document_hash: BytesN<32>,
    #[topic]
    pub admin: Address,
    pub reason: String,
}

/// Public interface for legal SAFT operations
pub trait LegalSAFTTrait {
    /// Store legal document hash (admin only)
    fn store_legal_hash(
        env: Env,
        admin: Address,
        vault_id: u64,
        document_type: DocumentType,
        ipfs_cid: String,
        document_hash: BytesN<32>,
        jurisdiction: String,
        version: String,
        expires_at: Option<u64>,
    ) -> Result<(), LegalSAFTError>;

    /// Sign legal document (beneficiary)
    fn sign_legal_document(
        env: Env,
        beneficiary: Address,
        vault_id: u64,
        document_hash: BytesN<32>,
        signature: Bytes,
        message: String,
    ) -> Result<(), LegalSAFTError>;

    /// Check if all documents are signed
    fn are_all_documents_signed(env: Env, vault_id: u64) -> bool;

    /// Get legal document
    fn get_legal_document(env: Env, document_hash: BytesN<32>) -> Option<LegalDocument>;

    /// Get document signature
    fn get_document_signature(
        env: Env,
        beneficiary: Address,
        document_hash: BytesN<32>,
    ) -> Option<DocumentSignature>;

    /// Get vault legal documents
    fn get_vault_legal_documents(env: Env, vault_id: u64) -> Option<VaultLegalDocuments>;

    /// Revoke legal document (admin only)
    fn revoke_legal_document(
        env: Env,
        admin: Address,
        document_hash: BytesN<32>,
        reason: String,
    ) -> Result<(), LegalSAFTError>;
}
