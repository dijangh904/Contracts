use soroban_sdk::{Env, Address, vec, IntoVal};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient};
use vesting_vault::{VestingVault, VestingVaultClient, ClaimEvent};

#[test]
fn test_export_claims() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Setup a mock token
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
    let token = TokenClient::new(&env, &token_id);
    let stellar_asset = StellarAssetClient::new(&env, &token_id);

    // Initial supply to user1
    stellar_asset.mint(&user1, &1000i128);
    
    // Proper way to call transfer in tests (not addr.invoke_stellar_asset_contract)
    stellar_asset.transfer(&user1, &user2, &100i128);
    assert_eq!(token.balance(&user2), 100);

    // Simulate claims via the contract
    client.claim(&user1, &1u32, &50i128);
    client.claim(&user2, &2u32, &30i128);

    // Proper way to call contract methods (not addr.invoke_contract)
    let claims = client.get_all_claims();
    assert_eq!(claims.len(), 2);

    let user1_claims = client.get_claims_by_user(&user1);
    assert_eq!(user1_claims.len(), 1);
    assert_eq!(user1_claims.get(0).unwrap().beneficiary, user1);
    assert_eq!(user1_claims.get(0).unwrap().amount, 50);
}