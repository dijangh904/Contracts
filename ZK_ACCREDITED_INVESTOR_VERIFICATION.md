# Zero-Knowledge Accredited Investor Verification

## Overview

This implementation addresses Issue #209 by integrating a ZK-Verifier function that enables privacy-preserving accredited investor verification. Instead of storing plain-text KYC status, the contract verifies a ZK-SNARK proof submitted by the user, mathematically proving they meet the jurisdictional requirements for "Accredited Investor" without revealing their country of origin or net worth.

## Architecture

### Core Components

1. **ZK-SNARK Verifier**: Verifies zero-knowledge proofs for accredited investor status
2. **Nullifier System**: Prevents double-spending of accreditation proofs
3. **Verification Key Management**: Manages cryptographic keys for different circuits
4. **Accreditation Records**: Stores privacy-preserving accreditation status
5. **Jurisdiction Support**: Supports multiple jurisdictions with privacy preservation

### Key Features

- **Privacy-Preserving Verification**: Users prove accredited status without revealing sensitive financial information
- **Jurisdictional Privacy**: Country of origin is hashed and not revealed in plain text
- **Net Worth Privacy**: Actual net worth amounts are not stored on-chain
- **Double-Spending Prevention**: Nullifier system prevents proof reuse
- **Multi-Jurisdiction Support**: Supports different accredited investor criteria across jurisdictions
- **Expiry Management**: Accreditation proofs have configurable expiry times
- **Circuit Extensibility**: Architecture supports multiple verification circuits

## Implementation Details

### New Types

```rust
// ZK-SNARK proof structure
pub struct ZKProof {
    pub proof_data: Bytes,        // Serialized ZK-SNARK proof
    pub public_inputs: Vec<Bytes>, // Public inputs for the circuit
    pub nullifier: BytesN<32>,     // Nullifier to prevent double-spending
    pub circuit_id: BytesN<32>,    // Identifier for the verification circuit
    pub verification_key_hash: BytesN<32>, // Hash of the verification key
}

// Accredited Investor verification circuit public inputs
pub struct AccreditedInvestorInputs {
    pub jurisdiction_hash: BytesN<32>,  // Hash of jurisdiction (privacy-preserving)
    pub net_worth_threshold_met: bool, // Whether net worth threshold is met
    pub income_threshold_met: bool,     // Whether income threshold is met
    pub professional_certifications: bool, // Whether professional certifications exist
    pub timestamp: u64,                 // Proof generation timestamp
    pub expiry: u64,                   // When the accreditation proof expires
}

// Verification key metadata
pub struct VerificationKey {
    pub key_hash: BytesN<32>,
    pub circuit_type: BytesN<32>,      // "accredited_investor" or other types
    pub supported_jurisdictions: Vec<BytesN<32>>,
    pub created_at: u64,
    pub is_active: bool,
}

// Accreditation status record
pub struct AccreditationRecord {
    pub investor_address: Address,
    pub verified_at: u64,
    pub expires_at: u64,
    pub circuit_id: BytesN<32>,
    pub verification_key_hash: BytesN<32>,
    pub jurisdiction_hash: BytesN<32>,
}
```

### Storage Architecture

- **NULLIFIER_MAP**: Tracks used nullifiers to prevent double-spending
- **VERIFICATION_KEYS**: Stores verification key metadata
- **ACCR_RECORDS**: Stores accreditation records
- **SUPPORTED_CIRCUITS**: Maps circuit IDs to circuit types

### Key Functions

#### `verify_accredited_investor_proof(investor, proof)`
- Verifies a ZK-SNARK proof for accredited investor status
- Requires investor authentication
- Validates nullifier (prevents double-spending)
- Verifies circuit and jurisdiction support
- Checks proof expiry
- Stores accreditation record

#### `is_accredited_investor(investor)`
- Checks if an investor has valid accreditation
- Returns boolean based on current timestamp vs expiry

#### `create_vault_accredited_only(...)`
- Creates a vault with accredited investor requirement
- Verifies creator is accredited before vault creation
- Only accredited investors can create or receive these vaults

#### `transfer_vault_accredited(vault_id, from, to)`
- Transfers vault with accredited investor verification
- Requires both sender and receiver to be accredited

#### Admin Functions
- `add_zk_verification_key(admin, verification_key)`: Add verification key
- `add_zk_supported_circuit(admin, circuit_id, circuit_type)`: Add supported circuit

## ZK-SNARK Circuit Design

### Accredited Investor Circuit

The ZK-SNARK circuit verifies the following conditions without revealing sensitive data:

1. **Jurisdictional Requirements**: Verifies user meets accredited investor criteria for their jurisdiction
2. **Net Worth Threshold**: Confirms net worth meets minimum requirements (amount not revealed)
3. **Income Threshold**: Confirms income meets minimum requirements (amount not revealed)
4. **Professional Certifications**: Verifies professional certifications if applicable
5. **Timestamp Validity**: Ensures proof is generated within valid timeframe

### Privacy-Preserving Public Inputs

- **jurisdiction_hash**: Hash of jurisdiction identifier (prevents country revelation)
- **net_worth_threshold_met**: Boolean indicating if threshold met (no amount revealed)
- **income_threshold_met**: Boolean indicating if threshold met (no amount revealed)
- **professional_certifications**: Boolean for certification status
- **timestamp**: Proof generation time
- **expiry**: When accreditation expires

### Private Witness Inputs

- **Actual net worth amount** (never revealed)
- **Actual income amount** (never revealed)
- **Country of origin** (never revealed, only hash used)
- **Professional certification details** (never revealed)
- **User secret** for proof generation

## Security Features

### Double-Spending Prevention
- Nullifier system ensures each proof can only be used once
- Nullifiers are permanently tracked after use
- Prevents accreditation proof reuse

### Cryptographic Security
- ZK-SNARK proofs provide mathematical guarantee of statement validity
- Verification keys are managed securely
- Circuit integrity ensures only valid proofs pass verification

### Privacy Protection
- No sensitive financial data stored on-chain
- Jurisdiction information protected by hashing
- Accreditation status stored without revealing underlying criteria

### Expiry Management
- Accreditation proofs have configurable expiry times
- Automatic invalidation of expired proofs
- Fresh verification required for continued access

## Supported Jurisdictions

### United States (SEC)
- Net worth > $1M (excluding primary residence)
- Income > $200K (individual) or $300K (joint) for last 2 years
- Professional certifications (Series 7, 65, etc.)

### European Union
- Varies by member state
- Typically net worth or income thresholds
- Professional investor qualifications

### United Kingdom
- High net worth individuals
- Sophisticated investors
- Professional certifications

## Gas Cost Estimates

| Operation | Estimated Cost (XLM) |
|-----------|---------------------|
| Verify Accreditation Proof | ~0.04 XLM |
| Check Accreditation Status | ~0.005 XLM |
| Create Accredited-Only Vault | ~0.05 XLM |
| Transfer Accredited Vault | ~0.02 XLM |
| Add Verification Key | ~0.01 XLM |

*Note: These are estimates. Actual costs may vary based on ZK proof complexity.*

## Usage Examples

### Verifying Accredited Investor Status

```rust
// User generates ZK proof off-chain
let proof = generate_accredited_investor_proof(
    user_secret,
    net_worth,
    income,
    jurisdiction,
    professional_certs
);

// Submit proof to contract
contract.verify_accredited_investor_proof(
    investor_address,
    proof
);
```

### Creating Accredited-Only Vault

```rust
// Only accredited investors can create these vaults
let vault_id = contract.create_vault_accredited_only(
    accredited_investor,
    1000000,  // 1M tokens
    token_address,
    start_time,
    end_time,
    keeper_fee,
    is_revocable,
    is_transferable,
    step_duration
);
```

### Checking Accreditation Status

```rust
let is_accredited = contract.is_accredited_investor(investor_address);
let accreditation_record = contract.get_accreditation_record(investor_address);
```

## Integration with Existing Features

### Vesting Vaults
- Accredited investor verification can be required for specific vaults
- Maintains all existing vesting functionality
- Adds privacy layer to investor qualification

### Governance
- Admin functions for managing verification keys
- Multi-sig support for ZK circuit management
- Emergency pause compatibility

### Marketplace
- Accredited-only vault transfers
- Privacy-preserving investor verification
- Maintains marketplace functionality with accreditation checks

## Testing

The implementation includes comprehensive tests covering:

- **ZK Proof Verification**: Valid and invalid proof scenarios
- **Nullifier Management**: Double-spending prevention
- **Jurisdiction Support**: Multiple jurisdiction validation
- **Expiry Handling**: Time-based validation
- **Accredited-Only Operations**: Vault creation and transfers
- **Admin Functions**: Verification key and circuit management
- **Error Conditions**: Comprehensive error handling

## Future Enhancements

### Production ZK Integration
- Replace placeholder verification with actual ZK-SNARK library
- Implement real circuit compilation and verification
- Optimize gas costs for ZK operations

### Additional Circuits
- Qualified Buyer verification
- Institutional Investor verification
- Custom jurisdiction circuits

### Advanced Privacy Features
- Recursive proof composition
- Batch verification
- Privacy-preserving audit trails

## Compliance Considerations

### Regulatory Compliance
- Maintains audit capabilities while preserving privacy
- Jurisdiction-specific compliance requirements
- AML/KYC integration possibilities

### Data Privacy
- GDPR compliance through privacy-by-design
- No personal data stored on-chain
- Minimal data retention policies

## Security Considerations

### Current Limitations
- Placeholder ZK verification (requires production integration)
- Trust model for verification key management
- Circuit security depends on proper implementation

### Mitigations
- Comprehensive test coverage
- Clear separation of placeholder and production code
- Architecture designed for secure ZK integration
- Regular security audits recommended

## Conclusion

This implementation provides a robust foundation for Zero-Knowledge Accredited Investor Verification in the Vesting Vault contract. The privacy-preserving approach enables compliance with accredited investor requirements while protecting sensitive financial information.

The modular architecture allows for future enhancements and production ZK-SNARK integration, making Vesting-Vault a leader in privacy-preserving DeFi solutions for institutional and high-net-worth investors.

## Next Steps

1. **Production ZK Integration**: Replace placeholder with actual ZK-SNARK verification
2. **Circuit Development**: Develop production-ready accredited investor circuits
3. **Security Audit**: Comprehensive audit of ZK verification components
4. **Performance Testing**: Benchmark gas costs and optimize
5. **Documentation**: User guides and developer documentation
6. **Testnet Deployment**: Deploy to testnet for community feedback
7. **Mainnet Deployment**: Production deployment with proper governance
