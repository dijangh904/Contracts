# Deposit to Yield Adapter

A Soroban smart contract adapter that allows vault administrators to route aggregate unvested tokens (the vault's TVL) into whitelisted, low-risk lending protocols while tracking exact share ownership of external pools.

## Overview

The Deposit to Yield Adapter serves as a bridge between vesting vaults and external lending protocols, enabling:

- **Yield Generation**: Route unvested tokens to generate yield while tokens remain locked
- **Risk Management**: Only low-risk protocols (rating 1-2) can be whitelisted
- **Share Tracking**: Precise tracking of vault's ownership share in external pools
- **Admin Control**: Multi-sig admin controls for protocol whitelisting and operations

## Key Features

### Protocol Management
- **Whitelisting**: Admins can whitelist lending protocols with risk ratings
- **Risk Controls**: Only protocols with risk rating 1-2 (low risk) are allowed
- **Asset Support**: Each protocol specifies supported assets
- **Deposit Limits**: Minimum and maximum deposit limits per protocol

### Position Management
- **Vault Positions**: Track yield positions per vault
- **Share Tracking**: Exact share ownership in external pools
- **Yield Accumulation**: Track accumulated yield over time
- **Position Updates**: Support for additional deposits to existing positions

### Yield Operations
- **Yield Claiming**: Claim accumulated yield from positions
- **Position Withdrawal**: Withdraw principal and yield from protocols
- **Treasury Management**: Yield is routed to designated treasury address

## Architecture

### Data Structures

#### `LendingProtocol`
```rust
pub struct LendingProtocol {
    pub address: Address,           // Protocol contract address
    pub name: String,              // Human-readable name
    pub is_active: bool,          // Whether protocol is active
    pub risk_rating: u32,         // 1-5 risk rating (1 = lowest risk)
    pub supported_assets: Vec<Address>, // Supported token assets
    pub minimum_deposit: i128,     // Minimum deposit amount
    pub maximum_deposit: i128,     // Maximum deposit amount
}
```

#### `YieldPosition`
```rust
pub struct YieldPosition {
    pub protocol_address: Address,    // Protocol contract address
    pub asset_address: Address,       // Token asset address
    pub deposited_amount: i128,       // Total principal deposited
    pub shares: i128,                  // Shares received from protocol
    pub deposited_at: u64,            // Initial deposit timestamp
    pub last_yield_claim: u64,         // Last yield claim timestamp
    pub accumulated_yield: i128,      // Total yield accumulated
}
```

#### `VaultYieldSummary`
```rust
pub struct VaultYieldSummary {
    pub vault_id: u64,                 // Vault identifier
    pub total_deposited: i128,         // Total principal across all positions
    pub total_yield_accumulated: i128, // Total yield accumulated
    pub active_positions: Vec<YieldPosition>, // Active yield positions
}
```

### Storage Layout

- `Admin`: Contract administrator address
- `VestingContract`: Main vesting contract address
- `YieldTreasury`: Address for yield collection
- `WhitelistedProtocols`: Map of protocol_address → LendingProtocol
- `VaultPositions`: Map of vault_id → Vec<YieldPosition>
- `YieldSummary`: Map of vault_id → VaultYieldSummary
- `ProtocolCounter`: Counter for whitelisted protocols
- `IsPaused`: Contract pause state

## Core Functions

### Initialization
- `initialize(admin, vesting_contract, yield_treasury)`: Initialize the adapter

### Protocol Management
- `whitelist_protocol(admin, protocol)`: Add a lending protocol to whitelist
- `delist_protocol(admin, protocol_address)`: Remove a protocol from whitelist
- `get_whitelisted_protocols()`: Get all whitelisted protocols

### Yield Operations
- `deposit_to_yield(admin, vault_id, protocol_address, asset_address, amount)`: Deposit unvested tokens
- `claim_yield(admin, vault_id, protocol_address, asset_address)`: Claim accumulated yield
- `withdraw_position(admin, vault_id, protocol_address, asset_address)`: Withdraw full position

### Query Functions
- `get_vault_positions(vault_id)`: Get all yield positions for a vault
- `get_vault_yield_summary(vault_id)`: Get yield summary for a vault

### Admin Functions
- `set_pause(admin, paused)`: Pause/unpause contract operations

## Usage Example

### 1. Initialize the Adapter
```rust
let admin = Address::random(&env);
let vesting_contract = Address::random(&env);
let yield_treasury = Address::random(&env);

DepositToYieldAdapter::initialize(env, admin, vesting_contract, yield_treasury);
```

### 2. Whitelist a Lending Protocol
```rust
let protocol = LendingProtocol {
    address: protocol_address,
    name: String::from_str(&env, "USDC Lending Pool"),
    is_active: true,
    risk_rating: 1, // Low risk
    supported_assets: vec![&env, usdc_address],
    minimum_deposit: 1000,
    maximum_deposit: 1000000,
};

DepositToYieldAdapter::whitelist_protocol(env, admin, protocol);
```

### 3. Deposit Unvested Tokens
```rust
let shares_received = DepositToYieldAdapter::deposit_to_yield(
    env,
    admin,
    vault_id,
    protocol_address,
    usdc_address,
    5000, // 5000 USDC
);
```

### 4. Claim Yield
```rust
let claimed_yield = DepositToYieldAdapter::claim_yield(
    env,
    admin,
    vault_id,
    protocol_address,
    usdc_address,
);
```

### 5. Withdraw Position
```rust
let (principal, yield) = DepositToYieldAdapter::withdraw_position(
    env,
    admin,
    vault_id,
    protocol_address,
    usdc_address,
);
```

## Security Features

### Risk Management
- **Risk Rating Limits**: Only protocols with rating 1-2 can be whitelisted
- **Admin Controls**: All operations require admin authorization
- **Pause Functionality**: Contract can be paused in emergencies
- **Deposit Limits**: Per-protocol minimum and maximum deposit limits

### Access Control
- **Admin Authorization**: All admin functions require admin authentication
- **Protocol Validation**: Only whitelisted protocols can be used
- **Asset Validation**: Only supported assets can be deposited
- **Balance Checks**: Sufficient unvested balance required for deposits

## Integration with Vesting System

The adapter integrates with the main vesting contract to:

1. **Query Unvested Amounts**: Check available unvested tokens per vault and asset
2. **Transfer Tokens**: Move tokens from vesting contract to lending protocols
3. **Return Principal**: Return withdrawn principal to vesting contract
4. **Track TVL**: Maintain accurate Total Value Locked (TVL) calculations

## Events

The contract emits events for all major operations:

- `ProtocolWhitelisted`: When a protocol is added to whitelist
- `ProtocolDelisted`: When a protocol is removed from whitelist
- `DepositedToYield`: When tokens are deposited to a protocol
- `YieldClaimed`: When yield is claimed from a position
- `PositionWithdrawn`: When a position is fully withdrawn

## Testing

Run tests with:
```bash
cargo test
```

The test suite covers:
- Contract initialization
- Protocol whitelisting and validation
- Deposit operations
- Yield claiming
- Position withdrawals
- Pause functionality
- Multiple positions per vault
- Position accumulation

## Future Enhancements

Potential improvements for future versions:

1. **Auto-Compounding**: Automatically reinvest claimed yield
2. **Strategy Diversification**: Support for multiple protocols per asset
3. **Yield Optimization**: Algorithmic protocol selection based on APY
4. **Cross-Chain Support**: Support for protocols on other chains
5. **Liquidation Protection**: Automatic position rebalancing
6. **Performance Analytics**: Detailed yield performance metrics

## License

This contract is part of the vesting system and follows the same license terms.
