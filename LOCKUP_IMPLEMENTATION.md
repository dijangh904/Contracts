# Lock-Up Periods Implementation - Issue #211

## Overview

This implementation adds "Lock-Up Periods" functionality to the vesting vault system, enabling legal compliance requirements where tokens cannot be sold immediately after vesting. The system issues temporary "Wrapped" tokens that cannot be transferred to a DEX until the lock-up timer expires.

## Architecture

### Components

1. **LockupToken Contract** (`contracts/lockup_token/`)
   - Manages wrapped tokens with transfer restrictions
   - Handles token issuance and unwrapping after lock-up expiry
   - Maintains lock-up information and balances

2. **Enhanced VestingVault** (`contracts/vesting_vault/`)
   - Extended with lock-up configuration and management
   - Modified claim flow to issue wrapped tokens during lock-up periods
   - Integrated with existing security features

### Key Features

- **Configurable Lock-Up Periods**: Admin can set different lock-up durations per vesting schedule
- **Wrapped Token System**: Non-transferable tokens during lock-up period
- **Automatic Unwrapping**: Users can unwrap tokens after lock-up expiry
- **Security Integration**: Works with existing emergency pause, address whitelisting, and milestone features
- **Multiple Vesting Support**: Different lock-up periods for different vesting schedules

## Implementation Details

### LockupToken Contract

#### Core Functions

- `initialize(admin, underlying_token)`: Initialize the contract
- `configure_lockup(admin, vesting_id, lockup_duration_seconds)`: Set lock-up period
- `issue_wrapped_tokens(from, to, vesting_id, amount)`: Issue wrapped tokens (authorized only)
- `unwrap_tokens(user, vesting_id, amount)`: Unwrap tokens after lock-up expiry
- `wrapped_balance(user)`: Get wrapped token balance
- `is_unlocked(user, vesting_id)`: Check if tokens are unlocked

#### Storage Structure

```
LockupConfig {
    vesting_id: u32,
    lockup_duration_seconds: u64,
    created_at: u64,
}

LockupInfo {
    vesting_id: u32,
    amount: i128,
    locked_at: u64,
    unlock_time: u64,
    is_unwrapped: bool,
}
```

### VestingVault Integration

#### New Functions

- `configure_lockup(admin, vesting_id, lockup_duration_seconds, lockup_token_address)`: Configure lock-up
- `disable_lockup(admin, vesting_id)`: Disable lock-up for a vesting schedule
- `claim_with_lockup(user, vesting_id, amount)`: Enhanced claim with lock-up handling
- `is_user_unlocked(user, vesting_id)`: Check unlock status
- `get_user_unlock_time(user, vesting_id)`: Get unlock time

#### Enhanced Claim Flow

1. Check existing security features (emergency pause, address whitelisting, milestones)
2. Check if lock-up is configured for the vesting schedule
3. If lock-up is enabled:
   - Issue wrapped tokens via LockupToken contract
   - Emit lock-up claim event
4. If lock-up is disabled:
   - Process normal claim flow

## Usage Examples

### Basic Setup

```rust
// Initialize LockupToken contract
let lockup_token = LockupToken::initialize(env, admin, underlying_token);

// Configure lock-up for vesting schedule
vesting_vault.configure_lockup(
    env,
    admin,
    vesting_id: 1,
    lockup_duration_seconds: 86400, // 1 day
    lockup_token_address: lockup_token_address
);
```

### Claiming with Lock-Up

```rust
// User claims tokens during lock-up period
vesting_vault.claim_with_lockup(
    env,
    user,
    vesting_id: 1,
    amount: 1000
);

// User receives wrapped tokens (non-transferable)
let wrapped_balance = lockup_token.wrapped_balance(env, user);
assert_eq!(wrapped_balance, 1000);

// Check unlock status
let is_unlocked = lockup_token.is_unlocked(env, user, vesting_id: 1);
assert!(!is_unlocked); // Still locked
```

### Unwrapping After Lock-Up

```rust
// Wait for lock-up period to expire (or advance timestamp in tests)
env.ledger().set_timestamp(current_time + 86401);

// Unwrap tokens to get transferable tokens
lockup_token.unwrap_tokens(
    env,
    user,
    vesting_id: 1,
    amount: 1000
);

// Wrapped tokens are burned, user can now transfer underlying tokens
let wrapped_balance = lockup_token.wrapped_balance(env, user);
assert_eq!(wrapped_balance, 0);
```

### Multiple Vesting Schedules

```rust
// Configure different lock-up periods for different vesting schedules
vesting_vault.configure_lockup(env, admin, 1, 86400, lockup_token_address); // 1 day
vesting_vault.configure_lockup(env, admin, 2, 172800, lockup_token_address); // 2 days

// Claim from different schedules
vesting_vault.claim_with_lockup(env, user, 1, 1000); // 1-day lock-up
vesting_vault.claim_with_lockup(env, user, 2, 2000); // 2-day lock-up
```

## Security Considerations

### Authorization

- Only authorized minters (typically the vesting vault) can issue wrapped tokens
- Admin-only functions for lock-up configuration
- User authentication required for unwrapping tokens

### Integration with Existing Features

- **Emergency Pause**: Lock-up claims respect emergency pause functionality
- **Address Whitelisting**: Authorized payout addresses work with lock-up claims
- **Milestone Vesting**: Lock-up integrates with milestone-gated vesting
- **Privacy Claims**: Can be combined with zero-knowledge privacy features

### Attack Vectors Mitigated

- **Unauthorized Token Issuance**: Only authorized minters can issue wrapped tokens
- **Double Unwrapping**: Balance checks prevent double-spending
- **Timing Attacks**: Lock-up timestamps are set at issuance time
- **Front-running**: Claims are atomic with lock-up token issuance

## Testing

### Test Coverage

- Contract initialization and configuration
- Token issuance and unwrapping
- Lock-up period enforcement
- Authorization and security
- Multiple vesting schedules
- Integration with existing features

### Running Tests

```bash
# Test LockupToken contract
cd contracts/lockup_token
cargo test

# Test VestingVault lock-up integration
cd contracts/vesting_vault
cargo test --test lockup_periods
```

## Deployment

### Deployment Steps

1. Deploy LockupToken contract
2. Initialize with admin and underlying token address
3. Add VestingVault as authorized minter
4. Configure lock-up periods for vesting schedules
5. Update claim flow to use `claim_with_lockup`

### Configuration Parameters

- `lockup_duration_seconds`: Duration in seconds (e.g., 86400 = 1 day)
- `vesting_id`: Unique identifier for vesting schedule
- `lockup_token_address`: Address of deployed LockupToken contract

## Future Enhancements

### Potential Improvements

1. **Gradual Unlocking**: Implement partial unlocking over time
2. **Dynamic Lock-Up**: Variable lock-up periods based on vesting amount
3. **Cross-Chain Support**: Extend to multi-chain environments
4. **Governance Integration**: DAO-based lock-up period management
5. **Yield Generation**: Allow wrapped tokens to earn yield during lock-up

### Compatibility

- **Backward Compatible**: Existing claim functions remain unchanged
- **Optional Feature**: Lock-up can be enabled/disabled per vesting schedule
- **Gas Efficient**: Minimal additional gas cost for lock-up claims

## Conclusion

The lock-up periods implementation provides a robust solution for legal compliance requirements while maintaining the flexibility and security of the existing vesting system. The wrapped token approach ensures that tokens cannot be transferred during the lock-up period while preserving user ownership and enabling seamless unwrapping after the restriction expires.

The implementation is thoroughly tested, well-documented, and ready for production deployment with proper configuration and testing.
