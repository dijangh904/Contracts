use soroban_sdk::{Env, Vec, Address, Map, BytesN};
use crate::types::{ClaimEvent, AuthorizedPayoutAddress, AddressWhitelistRequest, Nullifier, Commitment, PathPaymentConfig, PathPaymentClaimEvent, LockupConfig, BeneficiaryReassignment, VetoVote, TokenSupplyInfo};

pub const CLAIM_HISTORY: &str = "CLAIM_HISTORY";
pub const AUTHORIZED_PAYOUT_ADDRESS: &str = "AUTHORIZED_PAYOUT_ADDRESS";
pub const PENDING_ADDRESS_REQUEST: &str = "PENDING_ADDRESS_REQUEST";

// Emergency pause storage keys
pub const AUDITORS: &str = "AUDITORS";
pub const AUDITOR_PAUSE_REQUESTS: &str = "AUDITOR_PAUSE_REQUESTS";
pub const EMERGENCY_PAUSE: &str = "EMERGENCY_PAUSE";

// Cross-project reputation storage keys
pub const REPUTATION_BRIDGE_CONTRACT: &str = "REPUTATION_BRIDGE_CONTRACT";
pub const REPUTATION_BONUS_APPLIED: &str = "REPUTATION_BONUS_APPLIED";

// Milestone vesting storage keys
pub const MILESTONE_CONFIGS: &str = "MILESTONE_CONFIGS";
pub const MILESTONE_STATUS: &str = "MILESTONE_STATUS";

// Zero-Knowledge Privacy Claims storage keys
pub const NULLIFIER_MAP: &str = "NULLIFIER_MAP";
pub const COMMITMENT_STORAGE: &str = "COMMITMENT_STORAGE";
pub const PRIVACY_CLAIM_HISTORY: &str = "PRIVACY_CLAIM_HISTORY";
pub const MERKLE_ROOTS: &str = "MERKLE_ROOTS";

// Stellar Horizon Path Payment Claim storage keys
pub const PATH_PAYMENT_CONFIG: &str = "PATH_PAYMENT_CONFIG";
pub const PATH_PAYMENT_CLAIM_HISTORY: &str = "PATH_PAYMENT_CLAIM_HISTORY";

// Lock-up period storage keys
pub const LOCKUP_CONFIGS: &str = "LOCKUP_CONFIGS";

// Beneficiary reassignment and governance veto storage keys
pub const BENEFICIARY_REASSIGNMENTS: &str = "BENEFICIARY_REASSIGNMENTS";
pub const VETO_VOTES: &str = "VETO_VOTES";
pub const TOKEN_SUPPLY_INFO: &str = "TOKEN_SUPPLY_INFO";
pub const REASSIGNMENT_COUNTER: &str = "REASSIGNMENT_COUNTER";
pub const GOVERNANCE_VETO_THRESHOLD: &str = "GOVERNANCE_VETO_THRESHOLD"; // Percentage (e.g., 5 for 5%)

// 48 hours in seconds
const TIMELOCK_DURATION: u64 = 172_800;

// 7 days in seconds for emergency pause
const EMERGENCY_PAUSE_DURATION: u64 = 604_800;

pub fn get_claim_history(e: &Env) -> Vec<ClaimEvent> {
    e.storage()
        .instance()
        .get(&CLAIM_HISTORY)
        .unwrap_or(Vec::new(e))
}

pub fn set_claim_history(e: &Env, history: &Vec<ClaimEvent>) {
    e.storage().instance().set(&CLAIM_HISTORY, history);
}

pub fn get_authorized_payout_address(e: &Env, beneficiary: &Address) -> Option<AuthorizedPayoutAddress> {
    e.storage()
        .instance()
        .get(&(AUTHORIZED_PAYOUT_ADDRESS, beneficiary))
}

pub fn set_authorized_payout_address(e: &Env, beneficiary: &Address, auth_address: &AuthorizedPayoutAddress) {
    e.storage().instance().set(&(AUTHORIZED_PAYOUT_ADDRESS, beneficiary), auth_address);
}

pub fn get_pending_address_request(e: &Env, beneficiary: &Address) -> Option<AddressWhitelistRequest> {
    e.storage()
        .instance()
        .get(& (PENDING_ADDRESS_REQUEST, beneficiary))
}

pub fn set_pending_address_request(e: &Env, beneficiary: &Address, request: &AddressWhitelistRequest) {
    e.storage().instance().set(&(PENDING_ADDRESS_REQUEST, beneficiary), request);
}

pub fn remove_pending_address_request(e: &Env, beneficiary: &Address) {
    e.storage().instance().remove(&(PENDING_ADDRESS_REQUEST, beneficiary));
}

pub fn get_timelock_duration() -> u64 {
    TIMELOCK_DURATION
}

// Emergency pause functions
pub fn get_auditors(e: &Env) -> Vec<Address> {
    e.storage()
        .instance()
        .get(&AUDITORS)
        .unwrap_or(Vec::new(e))
}

pub fn set_auditors(e: &Env, auditors: &Vec<Address>) {
    e.storage().instance().set(&AUDITORS, auditors);
}

pub fn get_auditor_pause_requests(e: &Env) -> Map<Address, crate::emergency::AuditorPauseRequest> {
    e.storage()
        .instance()
        .get(&AUDITOR_PAUSE_REQUESTS)
        .unwrap_or(Map::new(e))
}

pub fn set_auditor_pause_requests(e: &Env, requests: &Map<Address, crate::emergency::AuditorPauseRequest>) {
    e.storage().instance().set(&AUDITOR_PAUSE_REQUESTS, requests);
}

pub fn get_emergency_pause(e: &Env) -> Option<crate::emergency::EmergencyPause> {
    e.storage().instance().get(&EMERGENCY_PAUSE)
}

pub fn set_emergency_pause(e: &Env, pause: &crate::emergency::EmergencyPause) {
    e.storage().instance().set(&EMERGENCY_PAUSE, pause);
}

pub fn remove_emergency_pause(e: &Env) {
    e.storage().instance().remove(&EMERGENCY_PAUSE);
}

// Cross-project reputation functions
pub fn get_reputation_bridge_contract(e: &Env) -> Option<Address> {
    e.storage().instance().get(&REPUTATION_BRIDGE_CONTRACT)
}

pub fn set_reputation_bridge_contract(e: &Env, contract_address: &Address) {
    e.storage().instance().set(&REPUTATION_BRIDGE_CONTRACT, contract_address);
}

pub fn has_reputation_bonus_applied(e: &Env, beneficiary: &Address) -> bool {
    e.storage()
        .instance()
        .get(&(REPUTATION_BONUS_APPLIED, beneficiary))
        .unwrap_or(false)
}

pub fn set_reputation_bonus_applied(e: &Env, beneficiary: &Address) {
    e.storage().instance().set(&(REPUTATION_BONUS_APPLIED, beneficiary), &true);
}

// Milestone vesting functions
pub fn get_milestone_configs(e: &Env, vesting_id: u32) -> Option<Vec<u32>> {
    e.storage().instance().get(&(MILESTONE_CONFIGS, vesting_id))
}

pub fn set_milestone_configs(e: &Env, vesting_id: u32, milestones: &Vec<u32>) {
    e.storage().instance().set(&(MILESTONE_CONFIGS, vesting_id), milestones);
}

pub fn get_milestone_status(e: &Env, vesting_id: u32) -> Map<u32, bool> {
    e.storage()
        .instance()
        .get(&(MILESTONE_STATUS, vesting_id))
        .unwrap_or(Map::new(e))
}

pub fn set_milestone_status(e: &Env, vesting_id: u32, status: &Map<u32, bool>) {
    e.storage().instance().set(&(MILESTONE_STATUS, vesting_id), status);
}

pub fn get_emergency_pause_duration() -> u64 {
    EMERGENCY_PAUSE_DURATION
}

// Zero-Knowledge Privacy Claims functions

// Nullifier map functions - prevent double-spending in private claims
pub fn is_nullifier_used(e: &Env, nullifier: &Nullifier) -> bool {
    e.storage()
        .instance()
        .get(&(NULLIFIER_MAP, nullifier.clone()))
        .unwrap_or(false)
}

pub fn set_nullifier_used(e: &Env, nullifier: &Nullifier) {
    e.storage().instance().set(&(NULLIFIER_MAP, nullifier.clone()), &true);
}

// Commitment storage functions
pub fn get_commitment(e: &Env, commitment_hash: &BytesN<32>) -> Option<Commitment> {
    e.storage().instance().get(&(COMMITMENT_STORAGE, commitment_hash.clone()))
}

pub fn set_commitment(e: &Env, commitment_hash: &BytesN<32>, commitment: &Commitment) {
    e.storage().instance().set(&(COMMITMENT_STORAGE, commitment_hash.clone()), commitment);
}

pub fn mark_commitment_used(e: &Env, commitment_hash: &BytesN<32>) {
    if let Some(mut commitment) = get_commitment(e, commitment_hash) {
        commitment.is_used = true;
        set_commitment(e, commitment_hash, &commitment);
    }
}

// Privacy claim history functions
pub fn get_privacy_claim_history(e: &Env) -> Vec<crate::types::PrivacyClaimEvent> {
    e.storage()
        .instance()
        .get(&PRIVACY_CLAIM_HISTORY)
        .unwrap_or(Vec::new(e))
}

pub fn add_privacy_claim_event(e: &Env, event: &crate::types::PrivacyClaimEvent) {
    let mut history = get_privacy_claim_history(e);
    history.push_back(event.clone());
    e.storage().instance().set(&PRIVACY_CLAIM_HISTORY, &history);
}

// Merkle root management for ZK proofs
pub fn add_merkle_root(e: &Env, merkle_root: &BytesN<32>) {
    let mut roots = get_merkle_roots(e);
    roots.push_back(merkle_root.clone());
    e.storage().instance().set(&MERKLE_ROOTS, &roots);
}

pub fn get_merkle_roots(e: &Env) -> Vec<BytesN<32>> {
    e.storage()
        .instance()
        .get(&MERKLE_ROOTS)
        .unwrap_or(Vec::new(e))
}

pub fn is_valid_merkle_root(e: &Env, merkle_root: &BytesN<32>) -> bool {
    let roots = get_merkle_roots(e);
    roots.contains(merkle_root.clone())
}

// Stellar Horizon Path Payment Claim storage functions
pub fn get_path_payment_config(e: &Env) -> Option<PathPaymentConfig> {
    e.storage().instance().get(&PATH_PAYMENT_CONFIG)
}

pub fn set_path_payment_config(e: &Env, config: &PathPaymentConfig) {
    e.storage().instance().set(&PATH_PAYMENT_CONFIG, config);
}

pub fn get_path_payment_claim_history(e: &Env) -> Vec<PathPaymentClaimEvent> {
    e.storage()
        .instance()
        .get(&PATH_PAYMENT_CLAIM_HISTORY)
        .unwrap_or(Vec::new(e))
}

pub fn add_path_payment_claim_event(e: &Env, event: &PathPaymentClaimEvent) {
    let mut history = get_path_payment_claim_history(e);
    history.push_back(event.clone());
    e.storage().instance().set(&PATH_PAYMENT_CLAIM_HISTORY, &history);
}

// Lock-up period storage functions
pub fn get_lockup_config(e: &Env, vesting_id: u32) -> Option<LockupConfig> {
    e.storage().instance().get(&(LOCKUP_CONFIGS, vesting_id))
}

pub fn set_lockup_config(e: &Env, vesting_id: u32, config: &LockupConfig) {
    e.storage().instance().set(&(LOCKUP_CONFIGS, vesting_id), config);
}

pub fn remove_lockup_config(e: &Env, vesting_id: u32) {
    e.storage().instance().remove(&(LOCKUP_CONFIGS, vesting_id));
}

// Beneficiary reassignment and governance veto storage functions
pub fn get_reassignment_counter(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&REASSIGNMENT_COUNTER)
        .unwrap_or(0)
}

pub fn set_reassignment_counter(e: &Env, counter: u32) {
    e.storage().instance().set(&REASSIGNMENT_COUNTER, &counter);
}

pub fn get_beneficiary_reassignment(e: &Env, reassignment_id: u32) -> Option<BeneficiaryReassignment> {
    e.storage().instance().get(&(BENEFICIARY_REASSIGNMENTS, reassignment_id))
}

pub fn set_beneficiary_reassignment(e: &Env, reassignment_id: u32, reassignment: &BeneficiaryReassignment) {
    e.storage().instance().set(&(BENEFICIARY_REASSIGNMENTS, reassignment_id), reassignment);
}

pub fn remove_beneficiary_reassignment(e: &Env, reassignment_id: u32) {
    e.storage().instance().remove(&(BENEFICIARY_REASSIGNMENTS, reassignment_id));
}

pub fn get_veto_votes(e: &Env, reassignment_id: u32) -> Vec<VetoVote> {
    e.storage()
        .instance()
        .get(&(VETO_VOTES, reassignment_id))
        .unwrap_or(Vec::new(e))
}

pub fn set_veto_votes(e: &Env, reassignment_id: u32, votes: &Vec<VetoVote>) {
    e.storage().instance().set(&(VETO_VOTES, reassignment_id), votes);
}

pub fn add_veto_vote(e: &Env, reassignment_id: u32, vote: &VetoVote) {
    let mut votes = get_veto_votes(e, reassignment_id);
    votes.push_back(vote.clone());
    set_veto_votes(e, reassignment_id, &votes);
}

pub fn get_token_supply_info(e: &Env) -> TokenSupplyInfo {
    e.storage()
        .instance()
        .get(&TOKEN_SUPPLY_INFO)
        .unwrap_or(TokenSupplyInfo {
            total_supply: 0,
            last_updated: 0,
        })
}

pub fn set_token_supply_info(e: &Env, supply_info: &TokenSupplyInfo) {
    e.storage().instance().set(&TOKEN_SUPPLY_INFO, supply_info);
}

pub fn get_governance_veto_threshold(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get(&GOVERNANCE_VETO_THRESHOLD)
        .unwrap_or(5) // Default 5% threshold
}

pub fn set_governance_veto_threshold(e: &Env, threshold: u32) {
    e.storage().instance().set(&GOVERNANCE_VETO_THRESHOLD, &threshold);
}

// 7 days in seconds for governance veto period
const GOVERNANCE_VETO_PERIOD: u64 = 604_800;

pub fn get_governance_veto_period() -> u64 {
    GOVERNANCE_VETO_PERIOD
}
