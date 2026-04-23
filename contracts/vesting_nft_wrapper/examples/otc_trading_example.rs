use soroban_sdk::{Address, Env, String, U256, token};
use vesting_nft_wrapper::{VestingNFTWrapper, VestingNFTWrapperClient};
use vesting_contracts::{VestingContract, VestingContractClient};

pub fn create_otc_vesting_nft(
    env: &Env,
    vesting_contract: &Address,
    nft_wrapper: &Address,
    beneficiary: &Address,
    token_address: &Address,
    amount: i128,
    duration_months: u64,
) -> U256 {
    // Calculate vesting period
    let start_time = env.ledger().timestamp();
    let end_time = start_time + (duration_months * 30 * 24 * 60 * 60); // Approximate months
    
    // Create a transferable vesting vault
    let vesting_client = VestingContractClient::new(env, vesting_contract);
    
    // First, create the vault (this would normally be done by admin)
    let vault_id = vesting_client.create_vault_full(
        beneficiary.clone(),
        amount,
        start_time,
        end_time,
        0, // keeper_fee
        false, // is_revocable
        true,  // is_transferable - crucial for NFT wrapping
        0,     // step_duration
    );
    
    // Mint NFT that wraps this vault
    let nft_client = VestingNFTWrapperClient::new(env, nft_wrapper);
    let metadata = String::from_str(
        env,
        &format!("OTC Vesting - {} tokens over {} months", amount, duration_months)
    );
    
    // This would normally be called by the vesting contract automatically
    // For this example, we'll simulate it
    let token_id = nft_client.mint(
        beneficiary.clone(),
        vault_id,
        metadata,
    );
    
    token_id
}

pub fn simulate_otc_trade(
    env: &Env,
    nft_wrapper: &Address,
    from: &Address,
    to: &Address,
    token_id: U256,
    price: i128,
    payment_token: &Address,
) {
    let nft_client = VestingNFTWrapperClient::new(env, nft_wrapper);
    
    // Step 1: Buyer approves the NFT wrapper to spend their payment tokens
    let token_client = token::Client::new(env, payment_token);
    token_client.approve(to, nft_wrapper, &price);
    
    // Step 2: Transfer payment tokens to seller (in a real implementation, this would be atomic)
    token_client.transfer(to, from, &price);
    
    // Step 3: Transfer the NFT (and thus vesting rights) to buyer
    nft_client.transfer_from(from.clone(), to.clone(), token_id);
    
    // Now the buyer owns the vesting rights and can claim tokens as they vest
}

pub fn claim_from_nft_vesting(
    env: &Env,
    vesting_contract: &Address,
    nft_wrapper: &Address,
    owner: &Address,
    token_id: U256,
) -> i128 {
    let nft_client = VestingNFTWrapperClient::new(env, nft_wrapper);
    let vesting_client = VestingContractClient::new(env, vesting_contract);
    
    // Get the vault ID from the NFT
    let vault_id = nft_client.get_vault_id(token_id);
    
    // Verify the owner matches
    assert_eq!(nft_client.owner_of(token_id), *owner);
    
    // Claim available tokens
    let claimed = vesting_client.claim_tokens(vault_id, i128::MAX);
    
    claimed
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as TestAddress, testutils::Ledger as TestLedger};

    #[test]
    fn test_otc_vesting_nft_flow() {
        let env = Env::default();
        env.mock_all_auths();
        
        // Setup addresses
        let admin = Address::generate(&env);
        let vesting_contract = Address::generate(&env);
        let nft_wrapper = Address::generate(&env);
        let original_beneficiary = Address::generate(&env);
        let otc_buyer = Address::generate(&env);
        let token_address = Address::generate(&env);
        
        // Initialize contracts
        VestingContract::initialize(env.clone(), admin.clone(), 1000000);
        VestingNFTWrapper::initialize(env.clone(), admin.clone(), vesting_contract.clone());
        
        // Create NFT-wrapped vesting
        let token_id = create_otc_vesting_nft(
            &env,
            &vesting_contract,
            &nft_wrapper,
            &original_beneficiary,
            &token_address,
            1000,
            12, // 12 months
        );
        
        // Verify original ownership
        let nft_client = VestingNFTWrapperClient::new(&env, &nft_wrapper);
        assert_eq!(nft_client.owner_of(token_id), original_beneficiary);
        
        // Simulate OTC trade
        simulate_otc_trade(
            &env,
            &nft_wrapper,
            &original_beneficiary,
            &otc_buyer,
            token_id,
            500, // Trade price
            &token_address,
        );
        
        // Verify new ownership
        assert_eq!(nft_client.owner_of(token_id), otc_buyer);
        
        // Fast forward time to vest some tokens
        env.ledger().set_timestamp(env.ledger().timestamp() + (6 * 30 * 24 * 60 * 60)); // 6 months
        
        // New owner can now claim vested tokens
        let claimed = claim_from_nft_vesting(
            &env,
            &vesting_contract,
            &nft_wrapper,
            &otc_buyer,
            token_id,
        );
        
        assert!(claimed > 0);
    }
}
