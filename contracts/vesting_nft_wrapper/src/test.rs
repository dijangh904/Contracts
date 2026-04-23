#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Address, Env, String, U256};

    fn create_test_env() -> (Env, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let vesting_contract = Address::generate(&env);
        (env, admin, vesting_contract)
    }

    #[test]
    fn test_initialization() {
        let (env, admin, vesting_contract) = create_test_env();
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        assert_eq!(
            env.storage().instance().get(&DataKey::Admin).unwrap(),
            admin
        );
        assert_eq!(
            env.storage().instance().get(&DataKey::VestingContract).unwrap(),
            vesting_contract
        );
        assert_eq!(
            env.storage().instance().get(&DataKey::TokenCounter).unwrap(),
            0u64
        );
    }

    #[test]
    #[should_panic(expected = "Already initialized")]
    fn test_double_initialization() {
        let (env, admin, vesting_contract) = create_test_env();
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
    }

    #[test]
    fn test_mint() {
        let (env, admin, vesting_contract) = create_test_env();
        let user = Address::generate(&env);
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        // Mock the vesting contract authorization
        env.mock_all_auths();
        
        let metadata = String::from_str(&env, "Test Vesting NFT");
        let token_id = VestingNFTWrapper::mint(
            env.clone(),
            user.clone(),
            1u64, // vault_id
            metadata.clone(),
        );
        
        assert_eq!(token_id, U256::from_u64(1));
        
        let nft = VestingNFTWrapper::get_nft(&env, token_id);
        assert_eq!(nft.token_id, token_id);
        assert_eq!(nft.vault_id, 1);
        assert_eq!(nft.original_owner, user);
        assert_eq!(nft.current_owner, user);
        assert_eq!(nft.metadata, metadata);
        
        assert_eq!(VestingNFTWrapper::owner_of(env.clone(), token_id), user);
        assert_eq!(VestingNFTWrapper::get_vault_id(env.clone(), token_id), 1);
        assert_eq!(VestingNFTWrapper::total_supply(env.clone()), 1);
    }

    #[test]
    fn test_tokens_of_owner() {
        let (env, admin, vesting_contract) = create_test_env();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        env.mock_all_auths();
        
        let metadata = String::from_str(&env, "Test Vesting NFT");
        let token1 = VestingNFTWrapper::mint(env.clone(), user1.clone(), 1u64, metadata.clone());
        let token2 = VestingNFTWrapper::mint(env.clone(), user1.clone(), 2u64, metadata.clone());
        let token3 = VestingNFTWrapper::mint(env.clone(), user2.clone(), 3u64, metadata.clone());
        
        let user1_tokens = VestingNFTWrapper::tokens_of_owner(env.clone(), user1.clone());
        assert_eq!(user1_tokens.len(), 2);
        assert!(user1_tokens.contains(&token1));
        assert!(user1_tokens.contains(&token2));
        
        let user2_tokens = VestingNFTWrapper::tokens_of_owner(env.clone(), user2.clone());
        assert_eq!(user2_tokens.len(), 1);
        assert!(user2_tokens.contains(&token3));
    }

    #[test]
    fn test_approve() {
        let (env, admin, vesting_contract) = create_test_env();
        let owner = Address::generate(&env);
        let approved = Address::generate(&env);
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        env.mock_all_auths();
        
        let metadata = String::from_str(&env, "Test Vesting NFT");
        let token_id = VestingNFTWrapper::mint(env.clone(), owner.clone(), 1u64, metadata);
        
        VestingNFTWrapper::approve(env.clone(), approved.clone(), token_id);
        
        assert_eq!(
            VestingNFTWrapper::get_approved(env.clone(), token_id).unwrap(),
            approved
        );
    }

    #[test]
    #[should_panic(expected = "Cannot approve self")]
    fn test_approve_self() {
        let (env, admin, vesting_contract) = create_test_env();
        let owner = Address::generate(&env);
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        env.mock_all_auths();
        
        let metadata = String::from_str(&env, "Test Vesting NFT");
        let token_id = VestingNFTWrapper::mint(env.clone(), owner.clone(), 1u64, metadata);
        
        VestingNFTWrapper::approve(env.clone(), owner.clone(), token_id);
    }

    #[test]
    fn test_set_approval_for_all() {
        let (env, admin, vesting_contract) = create_test_env();
        let owner = Address::generate(&env);
        let operator = Address::generate(&env);
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        VestingNFTWrapper::set_approval_for_all(env.clone(), operator.clone(), true);
        
        assert!(VestingNFTWrapper::is_approved_for_all(
            env.clone(),
            owner.clone(),
            operator.clone()
        ));
        
        VestingNFTWrapper::set_approval_for_all(env.clone(), operator.clone(), false);
        
        assert!(!VestingNFTWrapper::is_approved_for_all(
            env.clone(),
            owner.clone(),
            operator.clone()
        ));
    }

    #[test]
    fn test_transfer_from() {
        let (env, admin, vesting_contract) = create_test_env();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        env.mock_all_auths();
        
        let metadata = String::from_str(&env, "Test Vesting NFT");
        let token_id = VestingNFTWrapper::mint(env.clone(), from.clone(), 1u64, metadata);
        
        VestingNFTWrapper::transfer_from(env.clone(), from.clone(), to.clone(), token_id);
        
        assert_eq!(VestingNFTWrapper::owner_of(env.clone(), token_id), to);
        
        let from_tokens = VestingNFTWrapper::tokens_of_owner(env.clone(), from);
        assert_eq!(from_tokens.len(), 0);
        
        let to_tokens = VestingNFTWrapper::tokens_of_owner(env.clone(), to);
        assert_eq!(to_tokens.len(), 1);
        assert!(to_tokens.contains(&token_id));
    }

    #[test]
    fn test_transfer_from_with_approval() {
        let (env, admin, vesting_contract) = create_test_env();
        let owner = Address::generate(&env);
        let approved = Address::generate(&env);
        let to = Address::generate(&env);
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        env.mock_all_auths();
        
        let metadata = String::from_str(&env, "Test Vesting NFT");
        let token_id = VestingNFTWrapper::mint(env.clone(), owner.clone(), 1u64, metadata);
        
        VestingNFTWrapper::approve(env.clone(), approved.clone(), token_id);
        
        // Now the approved address can transfer
        VestingNFTWrapper::transfer_from(env.clone(), owner.clone(), to.clone(), token_id);
        
        assert_eq!(VestingNFTWrapper::owner_of(env.clone(), token_id), to);
    }

    #[test]
    #[should_panic(expected = "Not token owner")]
    fn test_transfer_from_unauthorized() {
        let (env, admin, vesting_contract) = create_test_env();
        let owner = Address::generate(&env);
        let unauthorized = Address::generate(&env);
        let to = Address::generate(&env);
        
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        env.mock_all_auths();
        
        let metadata = String::from_str(&env, "Test Vesting NFT");
        let token_id = VestingNFTWrapper::mint(env.clone(), owner.clone(), 1u64, metadata);
        
        // Unauthorized user tries to transfer
        VestingNFTWrapper::transfer_from(env.clone(), owner.clone(), to.clone(), token_id);
    }
}
