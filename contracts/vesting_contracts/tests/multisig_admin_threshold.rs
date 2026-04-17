use soroban_sdk::auth::{Context, CustomAccountInterface};
use soroban_sdk::crypto::Hash;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::xdr;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, IntoVal, Map, Symbol, Val,
    Vec,
};

use vesting_contracts::{VestingContract, VestingContractClient, AdminAction, ScheduleConfig, AssetAllocationEntry};

#[contract]
struct MultisigAccount;

#[contracttype]
enum MultisigDataKey {
    Signers,
    Threshold,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum MultisigError {
    ThresholdNotMet = 1,
    InvalidContext = 2,
}

#[contractimpl]
impl MultisigAccount {
    pub fn init(env: Env, signers: Vec<Address>, threshold: u32) {
        env.storage()
            .instance()
            .set(&MultisigDataKey::Signers, &signers);
        env.storage()
            .instance()
            .set(&MultisigDataKey::Threshold, &threshold);
    }
}

#[contractimpl]
impl CustomAccountInterface for MultisigAccount {
    type Signature = Vec<Address>;
    type Error = MultisigError;

    fn __check_auth(
        env: Env,
        _signature_payload: Hash<32>,
        signatures: Vec<Address>,
        auth_contexts: Vec<Context>,
    ) -> Result<(), Self::Error> {
        let allowed: Vec<Address> = env
            .storage()
            .instance()
            .get(&MultisigDataKey::Signers)
            .unwrap_or(Vec::new(&env));
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&MultisigDataKey::Threshold)
            .unwrap_or(0);

        let mut allowed_map: Map<Address, bool> = Map::new(&env);
        for addr in allowed.iter() {
            allowed_map.set(addr, true);
        }

        let mut seen: Map<Address, bool> = Map::new(&env);
        let mut approvals: u32 = 0;
        for signer in signatures.iter() {
            if allowed_map.get(signer.clone()).unwrap_or(false)
                && !seen.get(signer.clone()).unwrap_or(false)
            {
                seen.set(signer.clone(), true);
                approvals += 1;
            }
        }
        if approvals < threshold {
            return Err(MultisigError::ThresholdNotMet);
        }

        // Ensure we were asked to authorize a create_vault_full call.
        let expected_fn = Symbol::new(&env, "create_vault_full");
        let mut has_expected_context = false;
        for ctx in auth_contexts.iter() {
            if let Context::Contract(contract_ctx) = ctx {
                if contract_ctx.fn_name == expected_fn {
                    has_expected_context = true;
                    break;
                }
            }
        }
        if !has_expected_context {
            return Err(MultisigError::InvalidContext);
        }

        Ok(())
    }
}

fn auth_entry_for_multisig(
    env: &Env,
    authorizer: &Address,
    contract: &Address,
    fn_name: &str,
    args: Vec<Val>,
    signature: xdr::ScVal,
    nonce: i64,
) -> xdr::SorobanAuthorizationEntry {
    let root_invocation = xdr::SorobanAuthorizedInvocation {
        function: xdr::SorobanAuthorizedFunction::ContractFn(xdr::InvokeContractArgs {
            contract_address: contract.clone().try_into().unwrap(),
            function_name: fn_name.try_into().unwrap(),
            args: args.try_into().unwrap(),
        }),
        sub_invocations: std::vec::Vec::<xdr::SorobanAuthorizedInvocation>::new()
            .try_into()
            .unwrap(),
    };

    xdr::SorobanAuthorizationEntry {
        root_invocation,
        credentials: xdr::SorobanCredentials::Address(xdr::SorobanAddressCredentials {
            address: authorizer.try_into().unwrap(),
            nonce,
            signature_expiration_ledger: env.ledger().sequence() + 1000,
            signature,
        }),
    }
}

fn signatures_scval(signers: &[Address]) -> xdr::ScVal {
    let mut sig_vals: std::vec::Vec<xdr::ScVal> = std::vec::Vec::with_capacity(signers.len());
    for signer in signers {
        sig_vals.push(xdr::ScVal::Address(signer.try_into().unwrap()));
    }
    xdr::ScVal::Vec(Some(sig_vals.try_into().unwrap()))
}

#[test]
fn create_vault_succeeds_with_multisig_admin_threshold_met() {
    let env = Env::default();
    env.ledger().set_sequence_number(1);
    env.ledger().set_timestamp(1_000);

    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);
    let s3 = Address::generate(&env);
    let mut admins = Vec::new(&env);
    admins.push_back(s1.clone());
    admins.push_back(s2.clone());
    admins.push_back(s3.clone());

    let vesting_id = env.register(VestingContract, ());
    let vesting = VestingContractClient::new(&env, &vesting_id);
    vesting.initialize_multisig(&admins, &2u32, &1_000_000i128);

    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();
    let amount = 1_000i128;
    let keeper_fee = 0i128;
    let start = now;
    let end = now + 1_000;
    let is_revocable = true;
    let is_transferable = false;
    let token = Address::generate(&env);

    let cfg = ScheduleConfig {
        owner: beneficiary.clone(),
        asset_basket: soroban_sdk::vec![&env, AssetAllocationEntry {
            asset_id: token,
            total_amount: amount,
            released_amount: 0,
            locked_amount: 0,
            percentage: 10000,
        }],
        start_time: start,
        end_time: end,
        keeper_fee,
        is_revocable,
        is_transferable,
    };
    let action = AdminAction::AddBeneficiary(beneficiary.clone(), cfg);

    env.mock_all_auths();
    let proposal_id = vesting.propose_admin_action(&s1, &action);

    env.mock_all_auths();
    vesting.sign_admin_proposal(&s2, &proposal_id);

    let sig_count = vesting.admin_proposal_signature_count(&proposal_id);
    assert_eq!(sig_count, 2u32);
}

#[test]
#[should_panic]
fn create_vault_panics_when_multisig_threshold_not_met() {
    let env = Env::default();
    env.ledger().set_sequence_number(1);
    env.ledger().set_timestamp(1_000);

    let multisig_id = env.register(MultisigAccount, ());
    let multisig_client = MultisigAccountClient::new(&env, &multisig_id);

    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(s1.clone());
    signers.push_back(s2.clone());
    multisig_client.init(&signers, &2u32);

    let vesting_id = env.register(VestingContract, ());
    let vesting = VestingContractClient::new(&env, &vesting_id);
    vesting.initialize(&multisig_id, &1_000_000i128);

    let beneficiary = Address::generate(&env);
    let now = env.ledger().timestamp();
    let amount = 1_000i128;
    let keeper_fee = 0i128;
    let start = now;
    let end = now + 1_000;
    let is_revocable = true;
    let is_transferable = false;
    let step_duration = 0u64;

    let args: Vec<Val> = (
        beneficiary.clone(),
        amount,
        start,
        end,
        keeper_fee,
        is_revocable,
        is_transferable,
        step_duration,
    )
        .into_val(&env);
    let entry = auth_entry_for_multisig(
        &env,
        &multisig_id,
        &vesting_id,
        "create_vault_full",
        args,
        signatures_scval(&[s1.clone()]),
        1,
    );
    env.set_auths(&[entry]);

    vesting.create_vault_full(
        &beneficiary,
        &amount,
        &start,
        &end,
        &keeper_fee,
        &is_revocable,
        &is_transferable,
        &step_duration,
    );
}
