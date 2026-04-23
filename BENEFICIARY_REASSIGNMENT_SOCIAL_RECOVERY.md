# Beneficiary Reassignment (Social Recovery / Inheritance)

## Overview

This implementation addresses Issue #207 by providing a comprehensive beneficiary reassignment system that enables social recovery and inheritance for investors who lose their private keys or pass away. The system requires 2/3 multi-sig approval from DAO Admin council to legally transfer an active vesting schedule to a new Stellar public key.

## Architecture

### Core Components

1. **DAO Council Management**: Manages authorized council members for reassignment approvals
2. **Reassignment Request System**: Creates and tracks beneficiary reassignment requests
3. **Multi-Signature Approval**: Requires configurable number of approvals (default 2/3)
4. **Social Proof Integration**: Supports various social recovery proof types
5. **Emergency Reassignment**: Bypasses normal approval process for emergencies
6. **Vesting Integration**: Seamlessly integrates with existing vesting schedules

### Key Features

- **DAO Governance**: Council-based approval system for reassignment requests
- **Multi-Sig Security**: Configurable approval threshold (2/3 multi-sig by default)
- **Social Recovery**: Multiple proof types (death certificate, lost keys, court order, etc.)
- **Emergency Override**: Emergency admin can bypass normal approval process
- **IPFS Integration**: Social proof documents stored on IPFS
- **Time-Based Controls**: Approval windows and expiry times
- **Rate Limiting**: Prevents excessive reassignment requests per vault

## Implementation Details

### New Types

```rust
// Social recovery proof types
pub enum SocialProofType {
    DeathCertificate,      // Death certificate
    LostKeys,            // Lost private keys
    CourtOrder,          // Court order for reassignment
    MultiSig,            // Multi-signature from trusted parties
    EmergencyContact,      // Emergency contact verification
}

// Reassignment request status
pub enum ReassignmentStatus {
    None,
    Pending(Vec<Address>), // List of required approvers
    Approved,             // All approvals received
    Rejected,             // Reassignment rejected
    Completed,            // Reassignment completed
}

// Beneficiary reassignment request
pub struct ReassignmentRequest {
    pub vault_id: u64,
    pub current_beneficiary: Address,
    pub new_beneficiary: Address,
    pub requested_at: u64,
    pub expires_at: u64,
    pub social_proof_type: SocialProofType,
    pub social_proof_hash: [u8; 32], // Hash of social proof document
    pub social_proof_ipfs: String,   // IPFS CID of social proof
    pub reason: String,
    pub status: ReassignmentStatus,
    pub approvals: Vec<Address>,     // Received approvals
    pub required_approvals: u32,    // Required number of approvals
}

// DAO admin council member
pub struct DAOMember {
    pub address: Address,
    pub joined_at: u64,
    pub is_active: bool,
    pub role: String, // "admin", "council", "recovery"
}

// Reassignment configuration
pub struct ReassignmentConfig {
    pub required_approvals: u32,        // Default: 2/3 multi-sig
    pub approval_window: u64,           // Time to approve (default: 7 days)
    pub emergency_enabled: bool,         // Emergency reassignment enabled
    pub social_proof_required: bool,     // Social proof required
    pub max_reassignments_per_vault: u32, // Limit reassignments
}
```

### Storage Architecture

- **REASSIGNMENT_REQUESTS**: Stores reassignment requests indexed by vault_id
- **DAO_MEMBERS**: Stores DAO council members and their roles
- **REASSIGNMENT_CONFIG**: Stores reassignment system configuration
- **VAULT_REASSIGNMENTS**: Tracks reassignment count per vault

### Key Functions

#### `initialize_beneficiary_reassignment(admin, initial_members, required_approvals, approval_window)`
- Initializes DAO council and reassignment system
- Sets up initial council members with roles
- Configures approval requirements and time windows

#### `create_reassignment_request(current_beneficiary, new_beneficiary, vault_id, social_proof_type, social_proof_hash, social_proof_ipfs, reason)`
- Creates reassignment request with social proof
- Requires current beneficiary authentication
- Validates vault status and reassignment limits
- Emits `ReassignmentRequested` event

#### `approve_reassignment(approver, vault_id)`
- DAO council member approves reassignment request
- Requires council member authentication
- Prevents duplicate approvals
- Auto-completes reassignment when sufficient approvals received
- Emits `ReassignmentApproved` event

#### `emergency_reassignment(emergency_admin, vault_id, new_beneficiary, emergency_reason, social_proof_type, social_proof_hash, social_proof_ipfs)`
- Emergency reassignment bypassing normal approval process
- Requires emergency admin privileges
- Immediate completion of reassignment
- Emits `EmergencyReassignment` event

#### `reassign_beneficiary(vault_id, new_beneficiary, social_proof_type, social_proof_hash, social_proof_ipfs, reason)`
- Main function to legally transfer vesting schedule
- Creates reassignment request and completes immediately
- Updates vault ownership and user vault indexes
- Emits `BeneficiaryReassigned` event

#### Query Functions
- `get_reassignment_status(vault_id)`: Get reassignment request status
- `get_active_council_members()`: Get active DAO council members
- `add_dao_member(admin, member_address, role)`: Add new council member

## Security Features

### Multi-Signature Protection
- Configurable approval threshold (2/3 by default)
- Prevents single points of failure
- Council member authentication required
- Approval expiration windows

### Social Recovery Security
- Multiple proof types supported
- IPFS integration for document storage
- Hash verification for document integrity
- Emergency override capabilities

### Access Control
- Role-based permissions (admin, council, recovery)
- Authentication requirements for all operations
- Audit trail through event emissions

### Rate Limiting
- Maximum reassignments per vault (configurable)
- Request expiration times
- Duplicate request prevention

## Integration with Existing Features

### Vesting Schedules
- Seamless integration with existing vesting system
- Claim functions check reassignment status
- Vault ownership transfer with proper indexing
- Maintains all existing vesting functionality

### Inheritance System
- Complements existing Dead-Man's Switch
- Different use cases (reassignment vs inactivity)
- Both systems can coexist
- Independent operation but shared vault state

### Governance Integration
- Works with existing multi-sig admin system
- DAO council separate from contract admins
- Emergency admin override capabilities
- Configurable approval parameters

## Social Proof Types

### Death Certificate
- Official death certificate
- Court-issued document
- Notarized proof of death
- Most common for inheritance scenarios

### Lost Private Keys
- Self-attestation of lost keys
- Police report of lost keys
- Affidavit from trusted parties
- Common for key loss scenarios

### Court Order
- Legal court order for reassignment
- Judge-signed document
- Official legal proceeding
- Used in disputed situations

### Multi-Signature
- Signatures from multiple trusted parties
- Family member attestations
- Legal representative signatures
- Professional service provider attestations

### Emergency Contact
- Emergency contact verification
- Trusted third-party confirmation
- Medical emergency documentation
- Used in urgent recovery scenarios

## Gas Cost Estimates

| Operation | Estimated Cost (XLM) |
|-----------|---------------------|
| Initialize DAO | ~0.03 XLM |
| Create Reassignment Request | ~0.02 XLM |
| Approve Reassignment | ~0.015 XLM |
| Emergency Reassignment | ~0.025 XLM |
| Check Reassignment Status | ~0.005 XLM |
| Add DAO Member | ~0.01 XLM |
| Complete Reassignment | ~0.02 XLM |

*Note: These are estimates. Actual costs may vary based on complexity.*

## Usage Examples

### Initialize DAO Council

```rust
// Initialize DAO with 3 council members requiring 2 approvals
contract.initialize_beneficiary_reassignment(
    env,
    admin_address,
    vec![member1, member2, member3],
    2, // required_approvals
    7 * 24 * 60 * 60, // 7 days approval window
)?;
```

### Create Reassignment Request

```rust
// Current beneficiary creates reassignment request
contract.create_reassignment_request(
    env,
    current_beneficiary,
    new_beneficiary,
    vault_id,
    SocialProofType::DeathCertificate,
    death_cert_hash,
    "QmDeathCert123".to_string(),
    "Beneficiary passed away - death certificate provided".to_string(),
)?;
```

### Approve Reassignment Request

```rust
// DAO council member approves reassignment
contract.approve_reassignment(
    env,
    council_member,
    vault_id,
)?;
```

### Emergency Reassignment

```rust
// Emergency admin bypasses normal approval process
contract.emergency_reassignment(
    env,
    emergency_admin,
    vault_id,
    new_beneficiary,
    "Critical emergency - beneficiary incapacitated".to_string(),
    SocialProofType::EmergencyContact,
    emergency_contact_hash,
    "QmEmergency456".to_string(),
)?;
```

### Direct Reassignment

```rust
// Complete reassignment immediately (for trusted scenarios)
contract.reassign_beneficiary(
    env,
    vault_id,
    new_beneficiary,
    SocialProofType::LostKeys,
    lost_keys_hash,
    "QmLostKeys789".to_string(),
    "Lost private keys - need immediate recovery".to_string(),
)?;
```

## Testing

The implementation includes comprehensive tests covering:

- **DAO Initialization**: Council setup and configuration
- **Request Creation**: Valid and invalid reassignment requests
- **Approval Process**: Single and multi-sig approval scenarios
- **Emergency Override**: Emergency reassignment functionality
- **Error Conditions**: Comprehensive error handling
- **Integration Tests**: Vesting schedule integration
- **Security Tests**: Authorization and validation checks

## Compliance Considerations

### Legal Validity
- Court orders provide legal authority for reassignment
- Death certificates are legally binding inheritance documents
- Multi-signature requirements ensure proper verification
- Emergency provisions for critical situations

### Regulatory Compliance
- KYC/AML integration possibilities
- Jurisdiction-specific requirements handling
- Audit trail through event emissions
- Time-based controls for compliance windows

### Data Privacy
- Social proof hashes stored on-chain
- Actual documents stored on IPFS
- Minimal personal data exposure
- Configurable privacy settings

## Security Considerations

### Current Limitations
- Emergency admin power concentration
- Social proof verification is on-chain only
- Council member management complexity
- Rate limiting bypass possibilities

### Mitigations
- Multi-sig requirements prevent single points of failure
- Time-based controls prevent rushed decisions
- Audit trails through comprehensive event logging
- Configurable security parameters

### Future Security
- Social proof verification oracles
- Advanced multi-sig schemes
- Zero-knowledge proof integration
- Cross-chain reassignment support

## Comparison with Existing Inheritance

| Feature | Dead-Man's Switch | Beneficiary Reassignment |
|----------|-------------------|----------------------|
| Trigger | Inactivity | Explicit request |
| Use Case | Key loss prevention | Key recovery/inheritance |
| Time Control | Fixed timer | Configurable window |
| Approval | Automatic | Multi-sig DAO |
| Emergency | N/A | Emergency override |
| Proof Required | N/A | Social proof |

## Future Enhancements

### Advanced Social Recovery
- Integration with identity verification services
- Cross-chain social proof recognition
- Automated proof verification
- Multi-factor authentication support

### Enhanced DAO Features
- Dynamic approval thresholds
- Voting power weighting
- Proposal-based reassignment system
- Cross-DAO coordination

### Integration Improvements
- Automatic vesting schedule updates
- Batch reassignment operations
- Cross-contract reassignment support
- Advanced audit and reporting

## Conclusion

This implementation provides a robust and secure beneficiary reassignment system that addresses Issue #207 requirements. The system balances security with accessibility, providing a legal framework for social recovery and inheritance while maintaining the integrity of the vesting system.

The DAO-governed multi-sig approach ensures that reassignment decisions are made collectively, while emergency provisions provide necessary flexibility for critical situations. The integration with existing vesting schedules ensures seamless operation without disrupting current functionality.

## Next Steps

1. **Social Proof Verification**: Implement integration with identity verification services
2. **Advanced DAO Features**: Add voting power and proposal systems
3. **Cross-Chain Support**: Enable reassignments across different blockchain networks
4. **Enhanced Security**: Implement zero-knowledge proof verification
5. **Compliance Integration**: Add regulatory compliance checking
6. **Documentation**: User guides and legal documentation
7. **Testnet Deployment**: Deploy to testnet for community testing
8. **Mainnet Deployment**: Production deployment with legal review
