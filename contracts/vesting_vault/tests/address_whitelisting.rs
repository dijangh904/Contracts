#![cfg(test)]
use soroban_sdk::{Address, Env, Vec, IntoVal, Symbol, Val, Error};
use soroban_sdk::testutils::{Address as _, Ledger};
use vesting_vault::{VestingVault, VestingVaultClient};

fn setup() -> (Env, Address, VestingVaultClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VestingVault, ());
    let client = VestingVaultClient::new(&env, &contract_id);
    (env, contract_id, client)
}

#[test]
fn test_address_whitelisting() {
    let (env, contract_id, client) = setup();
    
    let beneficiary = Address::generate(&env);
    let hardware_wallet = Address::generate(&env);
    
    // Test 1: Set authorized payout address
    client.set_authorized_payout_address(&beneficiary, &hardware_wallet);
    
    // Check pending request
    let pending = client.get_pending_address_request(&beneficiary);
    assert!(pending.is_some(), "Pending request should exist");
    
    let request = pending.unwrap();
    assert!(request.beneficiary == beneficiary, "Beneficiary should match");
    assert!(request.requested_address == hardware_wallet, "Requested address should match");
    
    // Test 2: Try to confirm before timelock (should fail)
    // Assuming timelock is 172800 (48 hours)
    env.ledger().set_timestamp(request.requested_at + 172800 - 1000);
    
    let result = env.try_invoke_contract::<Val, Error>(
        &contract_id,
        &Symbol::new(&env, "confirm_auth_payout_addr"),
        (&beneficiary,).into_val(&env),
    );
    assert!(result.is_err(), "Should fail before timelock");
    
    // Test 3: Confirm after timelock
    env.ledger().set_timestamp(request.requested_at + 172800 + 1000);
    client.confirm_auth_payout_addr(&beneficiary);
    
    // Check authorized address
    let auth = client.get_authorized_payout_address(&beneficiary);
    assert!(auth.is_some(), "Authorized address should exist");
    
    let authorized = auth.unwrap();
    assert!(authorized.beneficiary == beneficiary, "Beneficiary should match");
    assert!(authorized.authorized_address == hardware_wallet, "Authorized address should match");
    assert!(authorized.is_active, "Should be active");
    
    // Test 4: Remove authorized address
    client.remove_authorized_payout_address(&beneficiary);
    let auth_after = client.get_authorized_payout_address(&beneficiary);
    assert!(auth_after.is_none(), "Authorized address should be removed");
}
