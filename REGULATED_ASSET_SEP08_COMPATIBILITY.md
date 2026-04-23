# Regulated Asset (SEP-08) Wrapper Compatibility

## Overview

This implementation addresses Issue #208 by providing comprehensive SEP-08 regulated asset wrapper compatibility for Vesting Vault contracts. The system ensures vaults can hold securities that require SEP-08 authorization, and can securely handle assets where issuers can freeze or claw back funds at the protocol layer without bricking the vault's internal accounting.

## Architecture

### Core Components

1. **Asset Regulation Registry**: Tracks which assets require SEP-08 compliance
2. **Authorization Management**: Manages SEP-08 authorizations for regulated assets
3. **Freeze/Clawback Protection**: Handles issuer-initiated freezes and clawbacks
4. **Vault Integration**: Seamlessly integrates with existing vesting system
5. **Compliance Framework**: Ensures regulatory compliance for regulated assets

### Key Features

- **SEP-08 Authorization**: Full support for Stellar's tokenized securities standard
- **Asset Registration**: Register regulated assets with compliance requirements
- **Authorization Validation**: Validate and consume authorizations for transfers
- **Freeze Protection**: Handle issuer-initiated asset freezes
- **Clawback Support**: Process issuer clawback requests
- **Accounting Safety**: Protect vault accounting from regulatory actions
- **Compliance Tracking**: Track compliance requirements and status

## Implementation Details

### New Types

```rust
// SEP-08 authorization status
pub enum AuthorizationStatus {
    None,
    Pending,
    Active,
    Revoked,
    Expired,
    Frozen,
}

// SEP-08 authorization data
pub struct SEP08Authorization {
    pub asset_id: Address,
    pub holder: Address,
    pub authorized_amount: i128,
    pub used_amount: i128,
    pub authorization_id: BytesN<32>,
    pub issued_at: u64,
    pub expires_at: u64,
    pub issuer: Address,
    pub status: AuthorizationStatus,
    pub compliance_flags: u32,
}

// Asset regulation metadata
pub struct AssetRegulation {
    pub asset_id: Address,
    pub is_regulated: bool,
    pub requires_authorization: bool,
    pub supports_freeze: bool,
    pub supports_clawback: bool,
    pub max_authorization_duration: u64,
    pub issuer: Address,
    pub regulation_version: u32,
    pub compliance_requirements: Vec<String>,
}

// Freeze/clawback event data
pub struct FreezeEvent {
    pub asset_id: Address,
    pub holder: Address,
    pub amount: i128,
    pub reason: String,
    pub timestamp: u64,
    pub issuer_signature: BytesN<32>,
}

pub struct ClawbackEvent {
    pub asset_id: Address,
    pub from_holder: Address,
    pub amount: i128,
    pub reason: String,
    pub timestamp: u64,
    pub issuer_signature: BytesN<32>,
}
```

### Storage Architecture

- **ASSET_REGULATIONS**: Stores asset regulation metadata indexed by asset_id
- **SEP08_AUTHORIZATIONS**: Stores SEP-08 authorizations indexed by authorization_id
- **FREEZE_EVENTS**: Stores freeze events indexed by (asset_id, holder, timestamp)
- **CLAWBACK_EVENTS**: Stores clawback events indexed by (asset_id, holder, timestamp)
- **VaultAuthorization**: Links vaults to their SEP-08 authorizations

### Key Functions

#### `register_regulated_asset(asset_id, issuer, requires_authorization, supports_freeze, supports_clawback, max_authorization_duration, compliance_requirements)`
- Registers a regulated asset with the system
- Sets compliance requirements and capabilities
- Emits `AssetRegistered` event

#### `create_authorization(asset_id, holder, authorized_amount, authorization_id, expires_at, issuer, compliance_flags)`
- Creates SEP-08 authorization for regulated asset
- Validates issuer authority and compliance requirements
- Emits `AuthorizationCreated` event

#### `validate_authorization(asset_id, holder, amount, authorization_id)`
- Validates SEP-08 authorization for transfers
- Checks expiration, status, and sufficient authorized amount
- Returns error if authorization invalid

#### `handle_freeze_event(asset_id, holder, amount, reason, issuer_signature)`
- Processes issuer-initiated asset freeze
- Updates all related authorizations to frozen status
- Emits `AssetFrozen` event

#### `handle_clawback_event(asset_id, from_holder, amount, reason, issuer_signature)`
- Processes issuer clawback of regulated assets
- Revokes all authorizations for affected holder/asset
- Emits `AssetClawback` event

#### `create_vault_regulated(...)`
- Creates vault with SEP-08 authorization support
- Validates authorization before vault creation
- Stores authorization reference with vault

#### `claim_tokens_regulated(vault_id, claim_amount, authorization_id)`
- Claims tokens with SEP-08 authorization validation
- Validates and consumes authorization amount
- Protects vault accounting from regulatory interference

## Security Features

### Authorization Security
- Cryptographic authorization IDs using 32-byte hashes
- Issuer signature verification for regulatory actions
- Time-based authorization expiration
- Authorization consumption tracking

### Asset Protection
- Freeze protection prevents unauthorized transfers
- Clawback support for issuer recovery actions
- Vault accounting isolation from regulatory events
- Compliance requirement enforcement

### Accounting Safety
- Separate tracking for regulated vs unregulated assets
- Protected vault internal accounting
- Event-based audit trails
- Reversible regulatory actions

## Integration with Existing Features

### Vesting Schedules
- Seamless integration with existing vesting system
- Authorization validation before token claims
- Protected vault accounting during regulatory actions
- Backward compatibility with unregulated assets

### Multi-Asset Support
- Per-asset regulation checking
- Mixed regulated/unregulated vault support
- Individual authorization validation per asset
- Comprehensive event tracking

### Compliance Framework
- Configurable compliance requirements per asset
- Regulatory event tracking and reporting
- Audit trail for all regulatory actions
- Time-based authorization controls

## SEP-08 Compliance

### Authorization Requirements
- KYC/AML verification for holders
- Accredited investor verification
- Jurisdictional compliance checking
- Transfer restrictions and limits
- Reporting requirements

### Regulatory Actions
- Asset freezes for compliance violations
- Clawbacks for regulatory requirements
- Authorization revocations
- Compliance reporting
- Emergency interventions

### Event Tracking
- All regulatory actions emit events
- Comprehensive audit trails
- Time-stamped regulatory events
- Issuer signature verification
- Cross-referenced authorization tracking

## Gas Cost Estimates

| Operation | Estimated Cost (XLM) |
|-----------|---------------------|
| Register Regulated Asset | ~0.025 XLM |
| Create Authorization | ~0.02 XLM |
| Validate Authorization | ~0.01 XLM |
| Handle Freeze Event | ~0.03 XLM |
| Handle Clawback Event | ~0.03 XLM |
| Create Regulated Vault | ~0.05 XLM |
| Claim Tokens Regulated | ~0.025 XLM |

*Note: These are estimates. Actual costs may vary based on complexity.*

## Usage Examples

### Register Regulated Asset

```rust
// Register a security token with SEP-08 compliance
contract.register_regulated_asset(
    env,
    security_token_address,
    token_issuer_address,
    true, // requires_authorization
    true, // supports_freeze
    true, // supports_clawback
    365 * 24 * 60 * 60, // 1 year max duration
    vec![
        "KYC required".to_string(),
        "Accredited investor only".to_string(),
        "US jurisdiction".to_string(),
    ],
)?;
```

### Create Authorization

```rust
// Create SEP-08 authorization for investor
contract.create_sep08_authorization(
    env,
    security_token_address,
    investor_address,
    1000000, // 10,000 tokens authorized
    BytesN::from_array([0x01; 32]), // authorization_id
    env.ledger().timestamp() + 365 * 24 * 60 * 60, // 1 year expiry
    token_issuer_address,
    0, // compliance flags
)?;
```

### Create Regulated Vault

```rust
// Create vault with SEP-08 authorization
let vault_id = contract.create_vault_regulated(
    env,
    investor_address,
    1000000,
    security_token_address,
    start_time,
    end_time,
    1000,
    true,
    true,
    86400, // 1 day step
    Some(BytesN::from_array([0x01; 32])), // authorization_id
)?;
```

### Claim with Authorization

```rust
// Claim tokens using SEP-08 authorization
let claimed_amount = contract.claim_tokens_regulated(
    env,
    vault_id,
    500000, // claim amount
    BytesN::from_array([0x01; 32]), // authorization_id
)?;
```

### Handle Regulatory Freeze

```rust
// Issuer freezes investor's tokens
contract.handle_asset_freeze(
    env,
    security_token_address,
    investor_address,
    250000, // amount to freeze
    "Regulatory compliance freeze".to_string(),
    BytesN::from_array([0x02; 32]), // issuer signature
)?;
```

## Testing

The implementation includes comprehensive tests covering:

- **Asset Registration**: Valid and invalid asset registration
- **Authorization Management**: Creation, validation, and consumption
- **Regulatory Actions**: Freeze and clawback event handling
- **Vault Integration**: Regulated vault creation and claiming
- **Error Conditions**: Comprehensive error handling
- **Security Tests**: Authorization validation and issuer verification

## Compliance Considerations

### Regulatory Requirements
- SEC compliance for security tokens
- KYC/AML integration points
- Accredited investor verification
- Transfer restrictions enforcement
- Reporting and audit capabilities

### Data Privacy
- Minimal on-chain sensitive data
- Off-chain document storage
- Authorization hash privacy
- Compliance requirement abstraction

### Audit Trails
- Complete event logging
- Time-stamped regulatory actions
- Authorization tracking and lifecycle
- Issuer signature verification

## Security Considerations

### Current Limitations
- Authorization management complexity
- Regulatory event processing overhead
- Multi-asset coordination challenges
- Cross-contract authorization synchronization

### Mitigations
- Efficient authorization validation
- Protected vault accounting
- Event-based audit trails
- Issuer signature verification
- Time-based authorization controls

### Future Security
- Zero-knowledge compliance proofs
- Advanced regulatory automation
- Cross-chain regulatory coordination
- Enhanced privacy features

## Comparison with Standard Assets

| Feature | Standard Assets | Regulated Assets (SEP-08) |
|----------|------------------|---------------------------|
| Transfer Requirements | Basic validation | Authorization validation |
| Regulatory Actions | None | Freeze/clawback support |
| Compliance | None | Full compliance framework |
| Audit Trail | Basic events | Comprehensive regulatory events |
| Asset Protection | Basic | Full regulatory protection |

## Future Enhancements

### Advanced SEP-08 Features
- Multi-authorization support
- Conditional authorizations
- Time-based transfer restrictions
- Advanced compliance automation
- Cross-chain regulatory coordination

### Integration Improvements
- Automated compliance checking
- Enhanced audit reporting
- Regulatory oracle integration
- Advanced privacy features
- Performance optimizations

## Conclusion

This implementation provides comprehensive SEP-08 regulated asset wrapper compatibility that addresses Issue #208 requirements. The system ensures vaults can securely hold securities while maintaining all regulatory compliance requirements.

The architecture protects vault accounting from regulatory interference while providing full support for issuer actions like freezes and clawbacks. The integration with existing vesting system ensures seamless operation for both regulated and unregulated assets.

## Next Steps

1. **SEP-08 Enhancement**: Implement advanced SEP-08 features
2. **Regulatory Integration**: Connect with regulatory oracles
3. **Compliance Automation**: Automated compliance checking
4. **Privacy Features**: Enhanced privacy protection
5. **Cross-Chain Support**: Multi-chain regulatory coordination
6. **Performance Optimization**: Gas cost reduction
7. **Documentation**: User guides and compliance documentation
8. **Testnet Deployment**: Deploy to testnet for regulatory validation
9. **Mainnet Deployment**: Production deployment with regulatory review
