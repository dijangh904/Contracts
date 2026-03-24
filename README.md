
## Deployed Contract
- **Network:** Stellar Testnet
- **Contract ID:** CD6OGC46OFCV52IJQKEDVKLX5ASA3ZMSTHAAZQIPDSJV6VZ3KUJDEP4D

## Gas Costs

| Operation | Estimated Cost (XLM) |
|-----------|---------------------|
| Create Vault | ~0.05 XLM |
| Claim | ~0.01 XLM |

*Note: These are estimated gas costs based on contract complexity. Actual costs may vary depending on network conditions and specific operation parameters.*


## Auto-Stake Feature

### How it works

Tokens stay locked inside the Vesting Vault at all times. When a beneficiary calls `auto_stake`, the vault makes a synchronous cross-contract call to a whitelisted staking contract, registering the vault's current locked balance as an active stake record. No token transfer occurs — the staking contract holds only the record, not the tokens.

### Staking lifecycle

```
Unstaked ──► auto_stake() ──► Staked
                                 │
                    manual_unstake() or revoke_vault()
                                 │
                              Unstaked
                                 │
                    (if revoked) treasury transfer
```

### Yield mechanics

Yield accrues on the staking contract against the beneficiary/vault pair. The beneficiary calls `claim_yield(vault_id)` on the vesting contract, which:

1. Calls `claim_yield_for(beneficiary, vault_id)` on the staking contract to get the accrued amount.
2. Transfers that amount from the staking contract's address to the beneficiary.
3. Resets `accumulated_yield` to zero.

Yield is claimable at any time while the vault is staked and has not been revoked.

### Revocation flow

1. Admin calls `revoke_vault(vault_id, treasury)`.
2. If the vault is currently staked, `do_unstake` is called first — the staking contract's record is cleared.
3. The vault is marked revoked in `RevokedVaults` storage.
4. All remaining unvested tokens are transferred to `treasury`.
5. The vault is frozen; no further claims or yield withdrawals are possible.

### Security assumptions

| What the vault trusts | What the vault verifies |
|---|---|
| Staking contract correctly records/clears stakes | Contract address is on the whitelist before any call |
| Staking contract holds yield tokens for payout | Yield transfer uses the staking contract as `from` address |
| Staking contract does not transfer vault tokens | No token transfer is initiated by the vault during staking |

### Integration guide

1. Deploy your staking contract implementing `stake_tokens`, `unstake_tokens`, `claim_yield_for`.
2. Call `add_staking_contract(staking_contract_id)` on the vesting contract (admin only).
3. Authorise the vesting contract address as a caller on the staking contract.
4. Beneficiaries can now call `auto_stake(vault_id, staking_contract_id)`.

To remove a staking contract from the whitelist: `remove_staking_contract(staking_contract_id)`.

### Error reference

| Error | Description |
|---|---|
| `AlreadyStaked` | Vault is already registered as a stake — unstake first |
| `NotStaked` | Operation requires the vault to be staked |
| `InsufficientBalance` | Vault has zero locked balance; nothing to stake |
| `UnauthorizedStakingContract` | Staking contract address is not whitelisted |
| `BeneficiaryRevoked` | Vault has been revoked; yield can no longer be claimed |
| `CrossContractCallFailed` | The cross-contract call to the staking contract failed |
| `UnstakeBeforeRevocationFailed` | Auto-unstake during revocation could not complete |
| `YieldClaimFailed` | The yield claim call to the staking contract failed |
| `Vault is irrevocable` | Cannot revoke a vault marked irrevocable |
