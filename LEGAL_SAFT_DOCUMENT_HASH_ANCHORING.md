# Legal SAFT Document Hash Anchoring

## Overview

This implementation addresses Issue #206 by providing a comprehensive legal document hash anchoring system for Vesting Vault contracts. The system enables admins to anchor IPFS CIDs of physical SAFT (Simple Agreement for Future Tokens) or Grant Agreement documents, and requires beneficiaries to cryptographically sign these hashes before the vesting clock starts.

## Architecture

### Core Components

1. **Legal Document Storage**: Stores metadata about anchored legal documents
2. **Document Signing System**: Manages beneficiary signatures on legal documents
3. **Vault Integration**: Links legal documents to specific vaults
4. **Vesting Clock Control**: Prevents token claims until all required documents are signed

### Key Features

- **IPFS Integration**: Stores IPFS CIDs of physical legal documents
- **Cryptographic Signing**: Beneficiaries sign document hashes on-chain
- **Multi-Document Support**: Vaults can require multiple legal documents
- **Document Expiry**: Optional expiry timestamps for time-sensitive documents
- **Jurisdiction Tracking**: Records legal jurisdiction for each document
- **Version Control**: Tracks document versions for compliance

## Implementation Details

### New Types

```rust
// Document types for legal agreements
pub enum DocumentType {
    SAFT,           // Simple Agreement for Future Tokens
    GrantAgreement,  // Grant Agreement
    PurchaseAgreement, // Purchase Agreement
    TokenWarrant,    // Token Warrant
    ConvertibleNote,  // Convertible Note
}

// Legal document metadata stored on-chain
pub struct LegalDocument {
    pub document_type: DocumentType,
    pub ipfs_cid: String,           // IPFS CID of document
    pub document_hash: BytesN<32>,   // SHA-256 hash of document
    pub admin_address: Address,       // Admin who anchored document
    pub anchored_at: u64,            // Timestamp when document was anchored
    pub expires_at: Option<u64>,     // Optional expiry timestamp
    pub jurisdiction: String,         // Legal jurisdiction
    pub version: String,             // Document version
}

// Beneficiary signature for legal document
pub struct DocumentSignature {
    pub beneficiary: Address,        // Beneficiary who signed
    pub document_hash: BytesN<32>,  // Hash of document being signed
    pub signature: Bytes,            // Cryptographic signature
    pub signed_at: u64,             // Timestamp when signed
    pub message: String,             // Optional message with signature
}

// Vault legal document association
pub struct VaultLegalDocuments {
    pub vault_id: u64,
    pub required_documents: Vec<BytesN<32>>, // List of required document hashes
    pub signed_documents: Vec<BytesN<32>>,   // List of signed document hashes
    pub all_documents_signed: bool,            // Whether all required documents are signed
    pub vesting_can_start: bool,               // Whether vesting can start
}
```

### Vault Structure Updates

The Vault struct has been enhanced with legal document tracking:

```rust
pub struct Vault {
    // ... existing fields ...
    pub requires_legal_signatures: bool,     // Whether legal signatures are required
    pub legal_documents_signed: bool,         // Whether all legal documents are signed
}
```

### Storage Architecture

- **LEGAL_DOCUMENTS**: Stores legal document metadata indexed by hash
- **DOCUMENT_SIGNATURES**: Stores beneficiary signatures indexed by (beneficiary, document_hash)
- **VAULT_LEGAL_DOCS**: Links vaults to their required and signed documents
- **DOCUMENT_INDEX**: Optional index for efficient document lookup

### Key Functions

#### `store_legal_hash(admin, vault_id, document_type, ipfs_cid, document_hash, jurisdiction, version, expires_at)`
- Admin function to anchor IPFS CID of physical legal document
- Validates IPFS CID format
- Stores document metadata on-chain
- Links document to specified vault
- Emits `LegalDocumentAnchored` event

#### `sign_legal_document(beneficiary, vault_id, document_hash, signature, message)`
- Beneficiary function to sign a legal document hash
- Requires beneficiary authentication
- Prevents duplicate signatures
- Updates vault status when all documents are signed
- Emits `DocumentSigned` and `AllDocumentsSigned` events

#### `create_vault_with_legal_requirements(...)`
- Creates a vault with legal document requirements
- Vesting clock only starts after all documents are signed
- Integrates with existing vault creation functionality

#### Claim Function Integration
Both `claim_tokens` and `claim_tokens_diversified` now check:
- If vault requires legal signatures
- If all required documents have been signed
- Prevents token claims if legal requirements are not met

## Security Features

### Document Integrity
- SHA-256 hashing ensures document integrity
- IPFS CID validation prevents invalid CIDs
- Document versioning tracks changes over time

### Signature Security
- Cryptographic signatures prevent forgery
- One signature per beneficiary per document
- Timestamped signatures create audit trail

### Access Control
- Admin-only document anchoring
- Beneficiary-only signing
- Proper authentication checks throughout

### Compliance Features
- Jurisdiction tracking for legal compliance
- Document expiry for time-sensitive agreements
- Version control for document updates

## Gas Cost Estimates

| Operation | Estimated Cost (XLM) |
|-----------|---------------------|
| Store Legal Hash | ~0.02 XLM |
| Sign Legal Document | ~0.015 XLM |
| Check Legal Status | ~0.005 XLM |
| Create Vault with Legal Requirements | ~0.05 XLM |
| Claim Tokens (with legal check) | ~0.01 XLM |

*Note: These are estimates. Actual costs may vary based on document complexity.*

## Usage Examples

### Admin Anchors Legal Document

```rust
// Admin anchors SAFT document
contract.store_legal_hash(
    admin_address,
    vault_id,
    DocumentType::SAFT,
    "QmXxx...".to_string(),           // IPFS CID
    document_hash,                      // SHA-256 hash
    "United States".to_string(),         // Jurisdiction
    "1.0".to_string(),               // Version
    Some(expiry_timestamp)              // Optional expiry
)?;
```

### Beneficiary Signs Document

```rust
// Beneficiary signs the SAFT
contract.sign_legal_document(
    beneficiary_address,
    vault_id,
    document_hash,
    signature,                          // Cryptographic signature
    "I agree to the terms of the SAFT".to_string()
)?;
```

### Create Vault with Legal Requirements

```rust
// Create vault that requires legal signatures
let vault_id = contract.create_vault_with_legal_requirements(
    beneficiary,
    1000000,                           // 1M tokens
    token_address,
    start_time,
    end_time,
    keeper_fee,
    is_revocable,
    is_transferable,
    step_duration,
    true                                // requires_legal_signatures
);
```

### Check Legal Status

```rust
// Check if all documents are signed
let all_signed = contract.are_legal_documents_signed(env, vault_id);

// Get vault legal documents status
let legal_status = contract.get_vault_legal_documents(env, vault_id);
```

## Integration with Existing Features

### Vesting Schedules
- Legal document signing is independent of vesting schedules
- Vesting clock only starts after legal requirements are met
- Maintains all existing vesting functionality

### Governance
- Admin functions for document management
- Multi-sig support for legal document operations
- Emergency pause compatibility

### Marketplace
- Legal document requirements transfer with vault ownership
- New owners must sign documents if required
- Maintains marketplace functionality with legal checks

## Document Types Supported

### SAFT (Simple Agreement for Future Tokens)
- Most common for token sales
- Standard terms for future token delivery
- Jurisdiction-specific variations

### Grant Agreement
- Used for ecosystem grants
- Milestone-based conditions
- Development requirements

### Purchase Agreement
- Direct token purchases
- Investment terms
- Transfer conditions

### Token Warrant
- Equity-like token rights
- Conversion terms
- Exercise conditions

### Convertible Note
- Debt-to-equity conversion
- Interest terms
- Maturity conditions

## Jurisdiction Support

### United States
- SEC compliance
- Accredited investor requirements
- State law considerations

### European Union
- MiCA regulation compliance
- AML/KYC requirements
- Member state variations

### United Kingdom
- FCA regulations
- Prospectus requirements
- Investor protections

### Asia-Pacific
- MAS (Singapore) compliance
- JFSA (Japan) requirements
- ASIC (Australia) regulations

## Testing

The implementation includes comprehensive tests covering:

- **Document Storage**: Valid and invalid document anchoring
- **Document Signing**: Successful signing and duplicate prevention
- **Vault Integration**: Legal requirement enforcement in claims
- **Multi-Document**: Multiple documents per vault
- **Expiry Handling**: Document expiration scenarios
- **Error Conditions**: Comprehensive error testing

## Compliance Considerations

### Legal Validity
- Documents stored as immutable hashes
- IPFS provides persistent storage
- Signatures create legally binding agreements

### Regulatory Compliance
- Jurisdiction tracking for regulatory requirements
- Document versioning for compliance updates
- Audit trail through event emissions

### Data Privacy
- Only document hashes stored on-chain
- IPFS stores actual documents off-chain
- Minimal personal data exposure

## Future Enhancements

### Advanced Document Types
- Equity agreements
- Revenue sharing agreements
- Governance token rights

### Batch Operations
- Batch document signing
- Bulk vault creation with legal requirements
- Multi-vault document templates

### Integration Features
- External legal oracle integration
- Automated document verification
- Cross-chain document recognition

## Security Considerations

### Current Limitations
- IPFS availability dependency
- Document hash collision possibility (theoretically)
- Signature verification is on-chain only

### Mitigations
- Multiple document storage locations
- SHA-256 collision resistance
- Comprehensive signature validation

### Future Security
- Document verification oracles
- Multi-signature document support
- Advanced cryptographic verification

## Conclusion

This implementation provides a robust foundation for Legal SAFT Document Hash Anchoring in Vesting Vault contracts. The system bridges the gap between physical legal agreements and smart contract execution, ensuring that beneficiaries have properly signed legal documents before accessing their vested tokens.

The architecture maintains all existing functionality while adding essential legal compliance features for real-world token distribution scenarios.

## Next Steps

1. **IPFS Integration**: Implement IPFS client for document retrieval
2. **Document Templates**: Create standard document templates
3. **Legal Oracle Integration**: Connect with legal verification services
4. **Advanced Signatures**: Support for multi-signature documents
5. **Compliance Automation**: Automated compliance checking
6. **Documentation**: User guides and legal documentation
7. **Testnet Deployment**: Deploy to testnet for legal validation
8. **Mainnet Deployment**: Production deployment with legal review
