#![cfg(test)]
use crate::{VestingContract, VestingContractClient, AdminAction, AdminProposal};
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn test_multisig_admin_proposal_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);

    // Setup 3 admins, quorum 2
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin3 = Address::generate(&env);
    let admins = vec![&env, admin1.clone(), admin2.clone(), admin3.clone()];
    client.initialize_multisig(&admins, &2u32, &1_000_000_000i128);

    // Propose to add a new admin
    let new_admin = Address::generate(&env);
    let action = AdminAction::AddAdmin(new_admin.clone());
    let proposal_id = client.propose_admin_action(&admin1, &action);

    // Only admin2 signs, admin1 already signed in propose
    client.sign_admin_proposal(&admin2, &proposal_id);

    // Check new admin is in the set
    let admin_set = client.get_admins();
    assert!(admin_set.contains(&new_admin));
}

#[test]
fn test_multisig_admin_proposal_insufficient_signatures() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VestingContract, ());
    let client = VestingContractClient::new(&env, &contract_id);

    // Setup 3 admins, quorum 3
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin3 = Address::generate(&env);
    let admins = vec![&env, admin1.clone(), admin2.clone(), admin3.clone()];
    client.initialize_multisig(&admins, &3u32, &1_000_000_000i128);

    // Propose to remove admin3
    let action = AdminAction::RemoveAdmin(admin3.clone());
    let _proposal_id = client.propose_admin_action(&admin1, &action);

    // Only admin1 signed (in propose). Quorum is 3. Action not executed.
    let admin_set = client.get_admins();
    assert!(admin_set.contains(&admin3));
}
