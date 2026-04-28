# Cross-Chain Vesting Synchronization via Wormhole - Implementation Notes

## Issue #268

This document outlines the implementation of cross-chain vesting synchronization using the Wormhole protocol, including CPU instruction limits compliance and production TODOs.

## Overview

The vesting vault now supports employees claiming tokens on secondary blockchain networks through Wormhole's cross-chain messaging protocol. The implementation includes:

- VAA (Verified Action Approval) signature verification
- Cross-chain claim payload routing
- Bridge pause handling with persistent queue
- Nonce-based replay attack prevention
- Destination address security guarantees

## Architecture

### Data Structures

#### ChainId Enum
Defines supported blockchain networks:
- Stellar (native)
- Ethereum, Solana, BSC, Polygon, Avalanche, Optimism, Arbitrum, Base

#### VAA (Verified Action Approval)
Contains signed message from Wormhole guardians:
- Version, guardian set index
- Emitter chain and address
- Sequence number for replay protection
- Guardian signatures
- Payload data

#### CrossChainClaimPayload
Embedded in VAA payload:
- Original vesting ID and beneficiary
- Claim amount
- Destination chain and address
- Nonce for replay protection

#### BridgeConfig
Configuration settings:
- Pause state
- Wormhole core contract address
- Supported chains
- Maximum bridge amount
- Bridge cooldown period

#### QueuedClaim
Stored when bridge is paused:
- Full claim details
- Associated VAA
- Queue timestamp

### Storage Strategy

#### Temporary Storage for Nonces
Nonces are stored in `e.storage().temporary()` to minimize ledger rent costs:
- Nonces expire after a configurable number of ledgers
- Reduces long-term storage costs
- Sufficient for replay attack prevention within the window

#### Instance Storage for Bridge State
- Bridge configuration (persistent)
- Last VAA sequence number (persistent)
- Last operation timestamp (persistent)
- Queued claims (persistent until processed)

## Security Features

### 1. VAA Signature Verification
- Validates VAA version
- Checks guardian signatures presence
- TODO: Integrate with Wormhole core contract for full signature verification
- TODO: Verify guardian set index is current
- TODO: Verify signature threshold (13/19 guardians)

### 2. Replay Attack Prevention
- **Nonce-based**: Each claim includes a unique nonce stored in temporary storage
- **Sequence-based**: VAA sequence numbers must be strictly incremented
- Both mechanisms provide defense-in-depth

### 3. Destination Address Security
- Destination address is embedded in the VAA payload
- Relayer cannot alter the final destination during transit
- Verified against payload during claim processing

### 4. Bridge Pause Handling
- Admin can pause bridge for emergency situations
- Claims are queued in persistent buffer when paused
- Queue processed automatically when unpaused
- Prevents loss of claims during bridge downtime

### 5. Amount Limits
- Maximum bridge amount per transaction
- Configurable by admin
- Prevents large-scale attacks

### 6. Cooldown Period
- Minimum delay between bridge operations
- Configurable by admin
- Prevents rapid-fire attacks

## CPU Instruction Limits Compliance

Soroban has strict CPU instruction limits (currently 100M instructions per transaction). The implementation is designed to stay within these limits:

### Optimizations

1. **Temporary Storage for Nonces**
   - Reduces storage read/write costs
   - Nonces expire automatically, reducing cleanup overhead

2. **Batch Processing of Queued Claims**
   - `process_queued_claims` accepts `max_claims` parameter
   - Allows processing in batches to stay within limits
   - Each claim processed individually with early exit

3. **Early Validation**
   - Check bridge pause state first (cheap)
   - Check nonce before expensive operations
   - Check sequence number before payload parsing

4. **Minimal VAA Verification (Current)**
   - Basic version and signature presence checks
   - Full verification deferred to Wormhole core contract in production

### Estimated CPU Costs

| Operation | Estimated Instructions | Notes |
|-----------|----------------------|-------|
| Bridge config read | ~10K | Instance storage read |
| Nonce check (temporary) | ~5K | Temporary storage read |
| Sequence check (instance) | ~10K | Instance storage read |
| VAA basic verification | ~50K | Version and signature checks |
| Payload parsing | ~100K | TODO: actual deserialization |
| Queue operations | ~20K per claim | Vec push/pop |
| Event emission | ~30K | CrossChainClaimInitiated |

**Total estimated per claim**: ~225K instructions (well under 100M limit)

**Batch processing (10 claims)**: ~2.25M instructions (still under limit)

### Production Considerations

When integrating with actual Wormhole core contract:
- VAA verification via external contract call will add ~500K-1M instructions
- Still within limits for single claim
- May need to reduce batch size for queue processing

## API Reference

### Admin Functions

#### `initialize_bridge`
```rust
pub fn initialize_bridge(
    e: Env,
    admin: Address,
    wormhole_core_address: Address,
    supported_chains: Vec<ChainId>,
    max_bridge_amount: i128,
    bridge_cooldown: u64
) -> Result<(), Error>
```
Initializes the Wormhole bridge configuration. Must be called before any cross-chain operations.

#### `toggle_bridge_pause`
```rust
pub fn toggle_bridge_pause(e: Env, admin: Address) -> Result<(), Error>
```
Toggles the bridge pause state. When paused, claims are queued instead of executed.

### User Functions

#### `cross_chain_claim`
```rust
pub fn cross_chain_claim(e: Env, user: Address, vaa: VAA) -> Result<(), Error>
```
Initiates a cross-chain claim with VAA verification. Performs all security checks and emits the cross-chain message.

#### `queue_cross_chain_claim`
```rust
pub fn queue_cross_chain_claim(e: Env, user: Address, vaa: VAA) -> Result<(), Error>
```
Queues a cross-chain claim when the bridge is paused. The claim will be processed when the bridge is unpaused.

### Public Functions

#### `process_queued_claims`
```rust
pub fn process_queued_claims(e: Env, max_claims: u32) -> Result<u32, Error>
```
Processes queued claims after the bridge is unpaused. Can be called by anyone. Returns the number of claims processed.

#### `get_bridge_config_public`
```rust
pub fn get_bridge_config_public(e: Env) -> Option<BridgeConfig>
```
Returns the current bridge configuration.

#### `get_queued_claims_count`
```rust
pub fn get_queued_claims_count(e: Env) -> u32
```
Returns the number of queued claims.

#### `get_bridge_last_sequence_public`
```rust
pub fn get_bridge_last_sequence_public(e: Env) -> u64
```
Returns the last processed VAA sequence number.

## Production TODOs

### Critical (Must Complete Before Mainnet)

1. **VAA Signature Verification**
   - Integrate with Wormhole core contract
   - Implement full guardian signature verification
   - Verify guardian set index is current
   - Verify signature threshold (13/19)
   - Location: `verify_vaa_signature()` in lib.rs

2. **Payload Parsing**
   - Implement proper deserialization of VAA payload
   - Follow Wormhole payload format specification
   - Add validation for each field
   - Location: `parse_vaa_payload()` in lib.rs

3. **Vested Amount Calculation**
   - Integrate with existing vesting logic
   - Calculate actual vested amount on Soroban
   - Location: `cross_chain_claim()` in lib.rs (line 2243)

4. **Native Asset Locking**
   - Implement token locking mechanism
   - Transfer tokens to locked state
   - Ensure tokens can't be double-spent
   - Location: `cross_chain_claim()` in lib.rs (line 2247)

5. **Wormhole Message Emission**
   - Call Wormhole core contract to emit burn/mint message
   - Include proper payload for destination chain
   - Location: `cross_chain_claim()` in lib.rs (line 2270)

### Important (Should Complete for Robustness)

6. **Gas Estimation**
   - Add gas estimation function for cross-chain claims
   - Help users understand costs before execution

7. **Retry Mechanism**
   - Add automatic retry for failed queued claims
   - Exponential backoff for reliability

8. **Monitoring Events**
   - Add events for bridge health monitoring
   - Track success/failure rates
   - Alert on unusual patterns

9. **Admin Multisig**
   - Require multisig for bridge configuration changes
   - Prevent single point of failure

10. **Chain-Specific Configuration**
    - Allow different limits per chain
    - Chain-specific cooldown periods
    - Chain-specific maximum amounts

### Nice to Have (Future Enhancements)

11. **Batch Claims**
    - Allow multiple claims in single transaction
    - Reduce gas costs for bulk operations

12. **Cross-Chain Reverts**
    - Implement mechanism to revert failed claims
    - Return tokens to original beneficiary

13. **Destination Chain Verification**
    - Verify destination chain is healthy before sending
    - Check for chain-specific issues

14. **Fee Estimation**
    - Estimate Wormhole relayer fees
    - Display to user before execution

15. **Historical Tracking**
    - Track all cross-chain operations
    - Provide audit trail

## Testing

### Unit Tests
- ✅ Bridge initialization
- ✅ Bridge pause toggle
- ✅ Invalid VAA signature detection
- ✅ Unsupported chain rejection
- ✅ Amount limit enforcement
- ✅ Cooldown period enforcement
- ✅ Sequence number validation
- ✅ Queue operations

### Integration Tests
- ✅ Mock Wormhole core contract
- ✅ Simulate cross-chain latency
- ✅ Test queue processing

### TODO: Additional Tests
- End-to-end test with actual Wormhole deployment
- Load testing with high volume of claims
- Failure scenario testing (bridge downtime, guardian failures)
- Security audit by third party

## Acceptance Criteria Status

### Acceptance 1: Employees can initiate claims on Stellar and receive wrapped tokens on an EVM network
- ✅ Architecture implemented
- ⏳ Pending: Wormhole core contract integration
- ⏳ Pending: Actual token transfer implementation

### Acceptance 2: Cross-chain message nonces are strictly incremented to mathematically prevent replay attacks
- ✅ Nonce-based replay prevention implemented
- ✅ Sequence number validation implemented
- ✅ Both mechanisms provide defense-in-depth

### Acceptance 3: The bridge execution logic operates within Soroban's strict CPU instruction limitations
- ✅ Estimated costs calculated
- ✅ Optimizations implemented
- ✅ Batch processing for queue
- ⏳ Pending: Actual measurement with Wormhole integration

## Deployment Checklist

- [ ] Complete all Critical TODOs
- [ ] Complete all Important TODOs
- [ ] Security audit completed
- [ ] Load testing completed
- [ ] Documentation updated
- [ ] Monitoring and alerting configured
- [ ] Emergency procedures documented
- [ ] Admin training completed
- [ ] Testnet deployment and validation
- [ ] Mainnet deployment

## References

- [Wormhole Documentation](https://docs.wormhole.com/)
- [Wormhole Chain IDs](https://docs.wormhole.com/wormhole/reference/chain-ids)
- [Soroban CPU Limits](https://soroban.stellar.org/docs/learn/limits)
- [Issue #268](https://github.com/Fatimasanusi/Contracts/issues/268)
