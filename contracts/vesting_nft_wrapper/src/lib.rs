#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, contractevent, Address, Env, String, Vec, U256, token,
};

mod vesting_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/vesting_contracts.wasm"
    );
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VestingNFT {
    pub token_id: U256,
    pub vault_id: u64,
    pub original_owner: Address,
    pub current_owner: Address,
    pub created_at: u64,
    pub metadata: String,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    VestingContract,
    TokenCounter,
    NFT(U256),
    OwnerTokens(Address),
    TokenApproval(U256),
    OperatorApproval(Address, Address),
}

#[contractevent]
pub struct MintEvent {
    #[topic]
    pub token_id: U256,
    #[topic]
    pub to: Address,
    #[topic]
    pub vault_id: u64,
}

#[contractevent]
pub struct TransferEvent {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    #[topic]
    pub token_id: U256,
}

#[contractevent]
pub struct ApprovalEvent {
    #[topic]
    pub owner: Address,
    #[topic]
    pub approved: Address,
    #[topic]
    pub token_id: U256,
}

#[contractevent]
pub struct ApprovalForAllEvent {
    #[topic]
    pub owner: Address,
    #[topic]
    pub operator: Address,
    #[topic]
    pub approved: bool,
}

#[contract]
pub struct VestingNFTWrapper;

#[contractimpl]
impl VestingNFTWrapper {
    pub fn initialize(env: Env, admin: Address, vesting_contract: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::VestingContract, &vesting_contract);
        env.storage().instance().set(&DataKey::TokenCounter, &0u64);
    }

    /// Mint a new NFT that wraps a vesting vault
    /// Only the vesting contract can call this
    pub fn mint(env: Env, to: Address, vault_id: u64, metadata: String) -> U256 {
        let vesting_addr: Address = env.storage().instance().get(&DataKey::VestingContract)
            .expect("Not initialized");
        
        // Only the vesting contract can mint
        vesting_addr.require_auth();

        let token_counter: u64 = env.storage().instance().get(&DataKey::TokenCounter)
            .expect("Token counter not found");
        let new_token_id = U256::from_u64(token_counter + 1);
        
        // Check if vault exists and is transferable
        let vesting_client = vesting_contract::Client::new(&env, &vesting_addr);
        let vault = vesting_client.get_vault(&vault_id);
        
        if !vault.is_transferable {
            panic!("Vault is not transferable");
        }
        
        if vault.owner != to {
            panic!("Vault owner mismatch");
        }

        let nft = VestingNFT {
            token_id: new_token_id,
            vault_id,
            original_owner: to.clone(),
            current_owner: to.clone(),
            created_at: env.ledger().timestamp(),
            metadata,
        };

        env.storage().instance().set(&DataKey::NFT(new_token_id), &nft);
        env.storage().instance().set(&DataKey::TokenCounter, &(token_counter + 1));

        // Add token to owner's collection
        let mut owner_tokens = env.storage().instance().get(&DataKey::OwnerTokens(to.clone()))
            .unwrap_or(Vec::new(&env));
        owner_tokens.push_back(new_token_id);
        env.storage().instance().set(&DataKey::OwnerTokens(to), &owner_tokens);

        MintEvent {
            token_id: new_token_id,
            to,
            vault_id,
        }.publish(&env);

        new_token_id
    }

    /// Transfer NFT and update vault ownership
    pub fn transfer_from(env: Env, from: Address, to: Address, token_id: U256) {
        from.require_auth();
        
        let nft = Self::get_nft(&env, token_id);
        
        if nft.current_owner != from {
            panic!("Not token owner");
        }

        // Check if sender is approved or owner
        if !Self::is_approved_or_owner(&env, from.clone(), token_id) {
            panic!("Not approved to transfer");
        }

        // Remove from old owner's collection
        let mut old_tokens = env.storage().instance().get(&DataKey::OwnerTokens(from.clone()))
            .unwrap_or(Vec::new(&env));
        let mut found = false;
        for i in 0..old_tokens.len() {
            if old_tokens.get(i).unwrap() == token_id {
                old_tokens.remove(i);
                found = true;
                break;
            }
        }
        if !found {
            panic!("Token not found in owner collection");
        }
        env.storage().instance().set(&DataKey::OwnerTokens(from), &old_tokens);

        // Add to new owner's collection
        let mut new_tokens = env.storage().instance().get(&DataKey::OwnerTokens(to.clone()))
            .unwrap_or(Vec::new(&env));
        new_tokens.push_back(token_id);
        env.storage().instance().set(&DataKey::OwnerTokens(to), &new_tokens);

        // Update NFT ownership
        let mut updated_nft = nft;
        updated_nft.current_owner = to.clone();
        env.storage().instance().set(&DataKey::NFT(token_id), &updated_nft);

        // Update vault ownership in the vesting contract
        Self::update_vault_ownership(&env, token_id, to.clone());

        // Clear any existing approval
        env.storage().instance().remove(&DataKey::TokenApproval(token_id));

        TransferEvent {
            from,
            to,
            token_id,
        }.publish(&env);
    }

    /// Approve an address to transfer a specific token
    pub fn approve(env: Env, to: Address, token_id: U256) {
        let nft = Self::get_nft(&env, token_id);
        nft.current_owner.require_auth();
        
        if to == nft.current_owner {
            panic!("Cannot approve self");
        }

        env.storage().instance().set(&DataKey::TokenApproval(token_id), &to);

        ApprovalEvent {
            owner: nft.current_owner,
            approved: to,
            token_id,
        }.publish(&env);
    }

    /// Set or unset approval for an operator to manage all tokens
    pub fn set_approval_for_all(env: Env, operator: Address, approved: bool) {
        let owner = env.current_contract_address();
        owner.require_auth();
        
        if operator == owner {
            panic!("Cannot set self as operator");
        }

        env.storage().instance().set(&DataKey::OperatorApproval(owner, operator), &approved);

        ApprovalForAllEvent {
            owner,
            operator,
            approved,
        }.publish(&env);
    }

    /// Get the owner of a token
    pub fn owner_of(env: Env, token_id: U256) -> Address {
        let nft = Self::get_nft(&env, token_id);
        nft.current_owner
    }

    /// Get the vault ID associated with a token
    pub fn get_vault_id(env: Env, token_id: U256) -> u64 {
        let nft = Self::get_nft(&env, token_id);
        nft.vault_id
    }

    /// Get NFT metadata
    pub fn token_metadata(env: Env, token_id: U256) -> String {
        let nft = Self::get_nft(&env, token_id);
        nft.metadata
    }

    /// Get all tokens owned by an address
    pub fn tokens_of_owner(env: Env, owner: Address) -> Vec<U256> {
        env.storage().instance().get(&DataKey::OwnerTokens(owner))
            .unwrap_or(Vec::new(&env))
    }

    /// Get total supply of NFTs
    pub fn total_supply(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::TokenCounter)
            .unwrap_or(0u64)
    }

    /// Check if an address is approved to transfer a token
    pub fn get_approved(env: Env, token_id: U256) -> Option<Address> {
        env.storage().instance().get(&DataKey::TokenApproval(token_id))
    }

    /// Check if an operator is approved for all tokens of an owner
    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        env.storage().instance().get(&DataKey::OperatorApproval(owner, operator))
            .unwrap_or(false)
    }

    /// Internal helper to get NFT
    fn get_nft(env: &Env, token_id: U256) -> VestingNFT {
        env.storage().instance().get(&DataKey::NFT(token_id))
            .expect("Token does not exist")
    }

    /// Internal helper to check if address is approved or owner
    fn is_approved_or_owner(env: &Env, spender: Address, token_id: U256) -> bool {
        let nft = Self::get_nft(env, token_id);
        
        if nft.current_owner == spender {
            return true;
        }

        if let Some(approved) = env.storage().instance().get(&DataKey::TokenApproval(token_id)) {
            if approved == spender {
                return true;
            }
        }

        if env.storage().instance().get(&DataKey::OperatorApproval(nft.current_owner, spender))
            .unwrap_or(false) {
            return true;
        }

        false
    }

    /// Update vault ownership in the vesting contract
    fn update_vault_ownership(env: &Env, token_id: U256, new_owner: Address) {
        let nft = Self::get_nft(env, token_id);
        let vesting_addr: Address = env.storage().instance().get(&DataKey::VestingContract)
            .expect("Not initialized");
        
        let vesting_client = vesting_contract::Client::new(env, &vesting_addr);
        
        // Authorize the NFT wrapper contract to transfer the vault
        // First authorize the marketplace transfer
        vesting_client.authorize_marketplace_transfer(&nft.vault_id, &env.current_contract_address());
        
        // Then complete the transfer to the new owner
        vesting_client.complete_marketplace_transfer(&nft.vault_id, &new_owner);
    }

    /// Admin function to update vesting contract address
    pub fn update_vesting_contract(env: Env, new_vesting_contract: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::VestingContract, &new_vesting_contract);
    }

    /// Internal helper to check admin
    fn require_admin(env: &Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();
    }

    /// Get detailed NFT information including vesting status
    pub fn get_nft_details(env: Env, token_id: U256) -> (VestingNFT, i128, i128, i128) {
        let nft = Self::get_nft(&env, token_id);
        let vesting_addr: Address = env.storage().instance().get(&DataKey::VestingContract)
            .expect("Not initialized");
        
        let vesting_client = vesting_contract::Client::new(&env, &vesting_addr);
        let (total_amount, released_amount, claimable, _) = vesting_client.get_vault_statistics(&nft.vault_id);
        
        (nft, total_amount, released_amount, claimable)
    }

    /// Batch transfer multiple NFTs
    pub fn batch_transfer_from(env: Env, from: Address, to: Address, token_ids: Vec<U256>) {
        from.require_auth();
        
        for token_id in token_ids.iter() {
            Self::transfer_from(env.clone(), from.clone(), to.clone(), token_id);
        }
    }

    /// Get all NFTs for a vault (should only be 1, but safety check)
    pub fn get_nfts_for_vault(env: Env, vault_id: u64) -> Vec<U256> {
        let total_supply = Self::total_supply(env.clone());
        let mut result = Vec::new(&env);
        
        for i in 1..=total_supply {
            let token_id = U256::from_u64(i);
            if let Ok(nft) = env.storage().instance().get::<_, VestingNFT>(&DataKey::NFT(token_id)) {
                if nft.vault_id == vault_id {
                    result.push_back(token_id);
                }
            }
        }
        
        result
    }

    /// Check if a vault is wrapped by an NFT
    pub fn is_vault_wrapped(env: Env, vault_id: u64) -> bool {
        let nfts = Self::get_nfts_for_vault(env, vault_id);
        !nfts.is_empty()
    }

    /// Emergency burn function for admin to destroy NFT and release vault
    pub fn emergency_burn(env: Env, token_id: U256) {
        Self::require_admin(&env);
        
        let nft = Self::get_nft(&env, token_id);
        let owner = nft.current_owner;
        
        // Remove from owner's collection
        let mut owner_tokens = env.storage().instance().get(&DataKey::OwnerTokens(owner))
            .unwrap_or(Vec::new(&env));
        let mut found = false;
        for i in 0..owner_tokens.len() {
            if owner_tokens.get(i).unwrap() == token_id {
                owner_tokens.remove(i);
                found = true;
                break;
            }
        }
        if found {
            env.storage().instance().set(&DataKey::OwnerTokens(owner), &owner_tokens);
        }
        
        // Remove NFT
        env.storage().instance().remove(&DataKey::NFT(token_id));
        
        // Clear any approvals
        env.storage().instance().remove(&DataKey::TokenApproval(token_id));
        
        // Note: The vault remains with the current owner, only the NFT wrapper is destroyed
    }
}
