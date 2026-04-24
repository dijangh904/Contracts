# Claim and Swap Implementation

## Overview

This implementation adds a `claim_and_swap` function to the vesting contracts that allows employees to claim their vested project tokens and atomically swap them for USDC via a DEX AMM in a single transaction. This addresses the common issue where employees need to sell tokens immediately to cover taxes.

## Key Features

### 1. Path Payment Configuration
- **`configure_path_payment`**: Admin function to set up the destination asset (USDC), minimum amounts, and swap paths
- **`disable_path_payment`**: Admin function to disable the feature
- **`get_path_payment_config`**: Public function to retrieve current configuration

### 2. Claim and Swap Function
- **`claim_and_swap`**: Main function that claims vested tokens and swaps them for USDC
- Includes comprehensive compliance checks (KYC, sanctions, AML, etc.)
- Supports multi-asset vaults with diversified allocations
- Atomic execution ensures either both claim and swap succeed or neither does

### 3. Simulation and Estimation
- **`simulate_claim_and_swap`**: Gas-free simulation to show expected amounts and execution feasibility
- Returns detailed information about source amount, estimated destination amount, and execution status

### 4. History and Events
- **`get_path_payment_claim_history`**: Retrieve all previous claim_and_swap transactions
- Comprehensive event emissions for transparency and off-chain tracking

## Implementation Details

### Data Structures

```rust
pub struct PathPaymentConfig {
    pub destination_asset: Address, // USDC or other stablecoin
    pub min_destination_amount: i128,
    pub path: Vec<Address>, // Path of assets for the swap
    pub enabled: bool,
}

pub struct PathPaymentClaimEvent {
    pub beneficiary: Address,
    pub source_amount: i128,
    pub destination_amount: i128,
    pub destination_asset: Address,
    pub timestamp: u64,
    pub vault_id: u64,
}

pub struct PathPaymentSimulation {
    pub source_amount: i128,
    pub estimated_destination_amount: i128,
    pub min_destination_amount: i128,
    pub path: Vec<Address>,
    pub can_execute: bool,
    pub reason: String,
    pub estimated_gas_fee: u64,
}
```

### Key Functions

#### `claim_and_swap(env, vault_id, min_destination_amount) -> Result<PathPaymentClaimEvent, Error>`

**Process Flow:**
1. Verify path payment is configured and enabled
2. Perform comprehensive compliance checks
3. Calculate claimable amounts across all vault assets
4. Validate minimum destination amount requirements
5. Update vault allocations (mark tokens as claimed)
6. Execute Stellar Path Payment (simplified implementation)
7. Transfer USDC to beneficiary
8. Record transaction and emit events

**Compliance Checks:**
- KYC verification and expiration
- Sanctions screening
- Jurisdiction restrictions
- Legal signature verification
- Document verification
- Tax compliance
- Whitelist/blacklist checks
- Geofencing restrictions
- Identity verification expiration
- PEP and sanctions list checks

#### `simulate_claim_and_swap(env, vault_id, min_destination_amount) -> PathPaymentSimulation`

Returns a detailed simulation showing:
- Available claimable amount
- Estimated USDC destination amount
- Whether execution is possible
- Reason for success/failure
- Estimated gas costs

### Error Handling

New error types added:
- `PathPaymentNotConfigured` (1000)
- `PathPaymentDisabled` (1001)
- `InsufficientLiquidity` (1002)
- `PathPaymentFailed` (1003)

### Security Features

1. **Admin Control**: Only admins can configure path payment settings
2. **Multi-sig Support**: Respects the contract's multi-signature admin system
3. **Emergency Pause**: Respects contract-wide pause functionality
4. **Atomic Execution**: Either both claim and swap succeed or neither does
5. **Compliance Integration**: Full integration with existing compliance framework

## Usage Examples

### Admin Setup
```rust
// Configure path payment to swap tokens for USDC
client.configure_path_payment(
    &admin,
    &usdc_address,           // Destination asset (USDC)
    &1000i128,              // Minimum USDC to receive
    &vec![intermediate_token] // Swap path
);
```

### Employee Claim and Swap
```rust
// Claim vested tokens and swap for USDC
let result = client.claim_and_swap(
    &vault_id,
    &Some(950i128) // Minimum USDC willing to accept
)?;
```

### Simulation
```rust
// Simulate without consuming gas
let simulation = client.simulate_claim_and_swap(
    &vault_id,
    &Some(950i128)
);

if simulation.can_execute {
    println!("Expected USDC: {}", simulation.estimated_destination_amount);
} else {
    println!("Cannot execute: {}", simulation.reason);
}
```

## Integration with Existing Features

The claim_and_swap functionality integrates seamlessly with:

1. **Diversified Vesting**: Supports multi-asset vaults
2. **Compliance Framework**: Uses existing compliance checks
3. **Certificate Registry**: Registers completion certificates
4. **NFT Minting**: Mints completion NFTs if configured
5. **Beneficiary Reassignment**: Respects reassignment status
6. **Legal SAFT**: Requires legal signatures if configured
7. **Emergency Features**: Respects emergency pause functionality

## Production Considerations

### Current Implementation Notes
- Uses simplified 1:1 conversion rate for demonstration
- In production, integrate with real DEX for price queries
- Consider implementing slippage protection
- Add gas optimization for large batches

### Future Enhancements
1. **Real DEX Integration**: Connect to actual Stellar DEX for price discovery
2. **Slippage Protection**: Add maximum slippage parameters
3. **Batch Processing**: Support batch claim_and_swap for multiple vaults
4. **Advanced Routing**: Implement optimal path finding for swaps
5. **Gas Optimization**: Optimize for lower transaction costs

## Testing

Comprehensive test suite included in `tests/claim_and_swap_test.rs`:

- Configuration and disable functionality
- Simulation accuracy
- Error handling for various edge cases
- History tracking
- Event emission verification

## Conclusion

This implementation provides a robust, secure, and compliant solution for employees to claim their vested tokens and immediately swap them for stablecoins. The atomic nature ensures tax obligations can be met efficiently while maintaining full compliance with the existing vesting framework.
