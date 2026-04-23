# Deposit to Yield Adapter - Deployment Guide

## Overview

This guide covers the deployment and integration of the Deposit to Yield Adapter with the existing vesting system.

## Prerequisites

1. **Soroban CLI**: Install the latest Soroban command-line tools
2. **Network Access**: Access to Stellar testnet/mainnet
3. **Admin Keys**: Multi-sig admin keys for the vesting system
4. **Token Addresses**: Addresses for tokens to be used in yield generation

## Deployment Steps

### 1. Build the Contract

```bash
# Navigate to the adapter directory
cd contracts/deposit_to_yield_adapter

# Build the contract
cargo build --target wasm32-unknown-unknown --release

# The compiled WASM will be at:
# target/wasm32-unknown-unknown/release/deposit_to_yield_adapter.wasm
```

### 2. Deploy to Network

```bash
# Deploy to testnet (replace with your network and keys)
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/deposit_to_yield_adapter.wasm \
  --source <ADMIN_KEY> \
  --network testnet

# Note the contract address returned
```

### 3. Initialize the Contract

```bash
# Initialize with admin, vesting contract, and yield treasury addresses
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  initialize \
  --admin <ADMIN_ADDRESS> \
  --vesting_contract <VESTING_CONTRACT_ADDRESS> \
  --yield_treasury <YIELD_TREASURY_ADDRESS>
```

### 4. Whitelist Lending Protocols

#### Example: Whitelist Compound USDC Pool

```bash
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  whitelist_protocol \
  --admin <ADMIN_ADDRESS> \
  --protocol '{
    "address": "<COMPOUND_USDC_ADDRESS>",
    "name": "Compound USDC Pool",
    "is_active": true,
    "risk_rating": 1,
    "supported_assets": ["<USDC_TOKEN_ADDRESS>"],
    "minimum_deposit": 1000,
    "maximum_deposit": 1000000
  }'
```

#### Example: Whitelist Aave USDT Pool

```bash
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  whitelist_protocol \
  --admin <ADMIN_ADDRESS> \
  --protocol '{
    "address": "<AAVE_USDT_ADDRESS>",
    "name": "Aave USDT Pool",
    "is_active": true,
    "risk_rating": 2,
    "supported_assets": ["<USDT_TOKEN_ADDRESS>"],
    "minimum_deposit": 500,
    "maximum_deposit": 500000
  }'
```

## Integration with Vesting System

### 1. Update Vesting Contract (Optional)

To enable seamless integration, you may want to update the main vesting contract to:

```rust
// Add to vesting contract's DataKey enum
YieldAdapter(Address),

// Add adapter address storage
env.storage().instance().set(&DataKey::YieldAdapter, &adapter_address);

// Add function to get unvested amount for adapter
pub fn get_unvested_amount(env: Env, vault_id: u64, asset_address: Address) -> i128 {
    let vault = Self::get_vault_internal(&env, vault_id);
    let mut unvested = 0i128;
    
    for allocation in vault.allocations.iter() {
        if allocation.asset_id == asset_address {
            unvested += allocation.total_amount - allocation.released_amount;
        }
    }
    
    unvested
}
```

### 2. Grant Token Permissions

Ensure the adapter contract has sufficient token allowances:

```bash
# Approve adapter to spend vesting contract's tokens
soroban contract invoke \
  --id <USDC_TOKEN_ADDRESS> \
  --source <VESTING_CONTRACT_KEY> \
  --network testnet \
  -- \
  approve \
  --from <VESTING_CONTRACT_ADDRESS> \
  --spender <ADAPTER_CONTRACT_ADDRESS> \
  --amount 1000000000
```

## Usage Examples

### Deposit Unvested Tokens to Yield

```bash
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  deposit_to_yield \
  --admin <ADMIN_ADDRESS> \
  --vault_id 1 \
  --protocol_address <COMPOUND_USDC_ADDRESS> \
  --asset_address <USDC_TOKEN_ADDRESS> \
  --amount 50000
```

### Claim Yield from Position

```bash
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  claim_yield \
  --admin <ADMIN_ADDRESS> \
  --vault_id 1 \
  --protocol_address <COMPOUND_USDC_ADDRESS> \
  --asset_address <USDC_TOKEN_ADDRESS>
```

### Withdraw Position

```bash
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  withdraw_position \
  --admin <ADMIN_ADDRESS> \
  --vault_id 1 \
  --protocol_address <COMPOUND_USDC_ADDRESS> \
  --asset_address <USDC_TOKEN_ADDRESS>
```

## Monitoring and Management

### Check Contract State

```bash
# Check if contract is paused
soroban contract read \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --network testnet \
  -- \
  is_paused

# Get vault positions
soroban contract read \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --network testnet \
  -- \
  get_vault_positions \
  --vault_id 1

# Get vault yield summary
soroban contract read \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --network testnet \
  -- \
  get_vault_yield_summary \
  --vault_id 1
```

### Emergency Controls

```bash
# Pause contract (emergency)
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  set_pause \
  --admin <ADMIN_ADDRESS> \
  --paused true

# Unpause contract
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  set_pause \
  --admin <ADMIN_ADDRESS> \
  --paused false

# Delist protocol
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  delist_protocol \
  --admin <ADMIN_ADDRESS> \
  --protocol_address <PROTOCOL_ADDRESS>
```

## Security Considerations

### 1. Multi-Sig Admin

- Ensure admin actions require multi-sig approval
- Use the existing vesting contract's multi-sig mechanism
- Test all admin actions with proper authorization

### 2. Risk Management

- Only whitelist protocols with risk rating 1-2
- Regularly audit protocol performance and risk
- Set appropriate deposit limits
- Monitor protocol health and delist if necessary

### 3. Token Security

- Use secure token approval mechanisms
- Monitor token balances regularly
- Implement withdrawal limits if needed
- Keep yield treasury secure

### 4. Protocol Integration

- Thoroughly test protocol integrations
- Monitor protocol contract upgrades
- Have backup protocols ready
- Implement emergency withdrawal procedures

## Best Practices

### 1. Yield Strategy

1. **Diversification**: Use multiple protocols per asset
2. **Risk Assessment**: Regular risk reviews of whitelisted protocols
3. **Yield Reinvestment**: Consider auto-compounding strategies
4. **Liquidity Management**: Maintain sufficient liquidity for withdrawals

### 2. Operations

1. **Regular Monitoring**: Daily checks of positions and yields
2. **Performance Tracking**: Track yield rates and performance
3. **Risk Alerts**: Set up alerts for protocol issues
4. **Documentation**: Keep detailed records of all operations

### 3. Governance

1. **Protocol Reviews**: Regular reviews of whitelisted protocols
2. **Risk Updates**: Update risk ratings based on performance
3. **Parameter Adjustments**: Adjust deposit limits as needed
4. **Transparency**: Public reporting of yield performance

## Troubleshooting

### Common Issues

1. **Insufficient Balance**: Ensure vesting contract has unvested tokens
2. **Protocol Not Whitelisted**: Check protocol whitelist status
3. **Asset Not Supported**: Verify asset is supported by protocol
4. **Contract Paused**: Check if contract is in paused state

### Debug Commands

```bash
# Check specific error details
soroban contract invoke \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --source <ADMIN_KEY> \
  --network testnet \
  -- \
  deposit_to_yield \
  --admin <ADMIN_ADDRESS> \
  --vault_id 1 \
  --protocol_address <PROTOCOL_ADDRESS> \
  --asset_address <ASSET_ADDRESS> \
  --amount 1000

# Check storage directly
soroban contract inspect \
  --id <ADAPTER_CONTRACT_ADDRESS> \
  --network testnet
```

## Maintenance

### Regular Tasks

1. **Weekly**: Check yield performance and protocol health
2. **Monthly**: Review risk ratings and protocol whitelist
3. **Quarterly**: Comprehensive strategy review
4. **Annually**: Full security audit

### Upgrade Procedures

1. **Protocol Upgrades**: Monitor and test protocol upgrades
2. **Contract Upgrades**: Plan for adapter contract upgrades
3. **Migration**: Have migration plans ready for major changes
4. **Testing**: Thoroughly test all upgrades on testnet

## Support

For deployment issues or questions:

1. Check the contract events for detailed error information
2. Review the test cases for expected behavior
3. Consult the main vesting contract documentation
4. Contact the development team for complex issues

---

**Note**: This guide assumes familiarity with Soroban smart contracts and the existing vesting system. Always test thoroughly on testnet before mainnet deployment.
