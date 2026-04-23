# Vesting NFT Wrapper

A Soroban smart contract that wraps vesting schedules into non-fungible tokens (NFTs), enabling over-the-counter (OTC) trading of locked token allocations. When the NFT is transferred, the claim rights for the underlying locked tokens automatically transfer to the new owner.

## Features

- **ERC-721 Compatible**: Standard NFT functionality with transfer, approve, and operator approval
- **Automatic Rights Transfer**: NFT ownership automatically transfers vesting claim rights
- **OTC Trading Ready**: Designed specifically for high-tier investors to trade locked allocations
- **Integration Ready**: Seamlessly integrates with existing vesting contracts
- **Batch Operations**: Support for batch transfers and multiple NFT management
- **Emergency Functions**: Admin controls for emergency situations

## Architecture

The system consists of two main components:

1. **VestingNFTWrapper**: The NFT contract that wraps vesting schedules
2. **VestingContract**: The existing vesting system that manages token locks and releases

### Key Components

- **VestingNFT**: Represents a wrapped vesting schedule
- **Marketplace Integration**: Uses existing marketplace transfer functions in vesting contract
- **Automatic Ownership Transfer**: NFT transfers automatically update vault ownership

## Core Functions

### NFT Management

```rust
// Mint a new NFT wrapping a vesting vault
pub fn mint(env: Env, to: Address, vault_id: u64, metadata: String) -> U256

// Transfer NFT and update vault ownership
pub fn transfer_from(env: Env, from: Address, to: Address, token_id: U256)

// Approve an address for specific token
pub fn approve(env: Env, to: Address, token_id: U256)

// Set operator approval for all tokens
pub fn set_approval_for_all(env: Env, operator: Address, approved: bool)
```

### Query Functions

```rust
// Get NFT owner
pub fn owner_of(env: Env, token_id: U256) -> Address

// Get vault ID from NFT
pub fn get_vault_id(env: Env, token_id: U256) -> u64

// Get detailed NFT information with vesting status
pub fn get_nft_details(env: Env, token_id: U256) -> (VestingNFT, i128, i128, i128)

// Get all tokens owned by an address
pub fn tokens_of_owner(env: Env, owner: Address) -> Vec<U256>
```

### Utility Functions

```rust
// Batch transfer multiple NFTs
pub fn batch_transfer_from(env: Env, from: Address, to: Address, token_ids: Vec<U256>)

// Check if vault is wrapped by NFT
pub fn is_vault_wrapped(env: Env, vault_id: u64) -> bool

// Emergency burn function
pub fn emergency_burn(env: Env, token_id: U256)
```

## Usage Example

### Creating an OTC Vesting NFT

```rust
use vesting_nft_wrapper::VestingNFTWrapperClient;
use vesting_contracts::VestingContractClient;

// 1. Create a transferable vesting vault
let vesting_client = VestingContractClient::new(&env, &vesting_contract);
let vault_id = vesting_client.create_vault_full(
    beneficiary,
    amount,
    start_time,
    end_time,
    0,      // keeper_fee
    false,  // is_revocable
    true,   // is_transferable - crucial for NFT wrapping
    0,      // step_duration
);

// 2. Mint NFT that wraps the vault
let nft_client = VestingNFTWrapperClient::new(&env, &nft_wrapper);
let token_id = nft_client.mint(
    beneficiary,
    vault_id,
    "OTC Vesting - 1000 tokens over 12 months".into(),
);
```

### OTC Trading

```rust
// 1. Buyer sends payment to seller (off-chain or separate contract)
token_client.transfer(&buyer, &seller, &price);

// 2. Seller transfers NFT to buyer
nft_client.transfer_from(&seller, &buyer, token_id);

// 3. Buyer now owns vesting rights and can claim
let claimed = vesting_client.claim_tokens(vault_id, i128::MAX);
```

## Integration with Vesting Contracts

The NFT wrapper integrates with existing vesting contracts through:

1. **Marketplace Authorization**: Uses `authorize_marketplace_transfer` to get transfer permissions
2. **Ownership Transfer**: Uses `complete_marketplace_transfer` to update vault ownership
3. **Vault Validation**: Ensures vaults are transferable before wrapping

## Security Considerations

- **Transferable Vaults Only**: Only vaults marked as `is_transferable` can be wrapped
- **Authorization Checks**: All transfers require proper authorization
- **Owner Validation**: Ensures vault ownership matches NFT ownership during mint
- **Emergency Controls**: Admin functions for emergency situations

## Events

The contract emits standard ERC-721 compatible events:

- `MintEvent`: When new NFT is minted
- `TransferEvent`: When NFT is transferred
- `ApprovalEvent`: When token is approved
- `ApprovalForAllEvent`: When operator is approved

## Testing

Run tests with:

```bash
cargo test --package vesting_nft_wrapper
```

## Deployment

1. Deploy the vesting contract first
2. Deploy the NFT wrapper contract
3. Initialize the NFT wrapper with the vesting contract address
4. Set the NFT wrapper as an authorized marketplace in the vesting contract

## License

This project is part of the Vesting Vault ecosystem and follows the same licensing terms.
