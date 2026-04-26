use soroban_sdk::{contracttype, contractevent, Address, Vec, Map, String, BytesN, Bytes};

#[contracttype]
#[derive(Clone)]
pub struct ClaimEvent {
    pub beneficiary: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub vesting_id: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct AuthorizedPayoutAddress {
    pub beneficiary: Address,
    pub authorized_address: Address,
    pub requested_at: u64,
    pub effective_at: u64,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct AddressWhitelistRequest {
    pub beneficiary: Address,
    pub requested_address: Address,
    pub requested_at: u64,
    pub effective_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct AuthorizedAddressSet {
    #[topic]
    pub beneficiary: Address,
    pub authorized_address: Address,
    pub effective_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct AddressWhitelistRequested {
    #[topic]
    pub beneficiary: Address,
    pub requested_address: Address,
    pub requested_at: u64,
    pub effective_at: u64,
}

// Milestone vesting types
#[contracttype]
#[derive(Clone)]
pub struct MilestoneConfig {
    pub vesting_id: u32,
    pub milestone_percentages: Vec<u32>, // Percentage for each milestone (e.g., [25, 25, 50])
    pub total_milestones: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct MilestoneStatus {
    pub vesting_id: u32,
    pub completed_milestones: Map<u32, bool>, // milestone_number -> completed
    pub last_completed: u32,
}

#[contractevent]
#[derive(Clone)]
pub struct MilestoneCompleted {
    #[topic]
    pub vesting_id: u32,
    pub milestone_number: u32,
    pub completed_at: u64,
}

// Simulation types
#[contracttype]
#[derive(Clone)]
pub struct ClaimSimulation {
    pub tokens_to_release: i128,
    pub estimated_gas_fee: u64,
    pub tax_withholding_amount: i128,
    pub net_amount: i128,
    pub can_claim: bool,
    pub reason: String,
}

// Reputation bridge types
#[contracttype]
#[derive(Clone)]
pub struct ReputationBonus {
    pub beneficiary: Address,
    pub cliff_reduction_months: u32,
    pub applied_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct ReputationBonusApplied {
    #[topic]
    pub beneficiary: Address,
    pub cliff_reduction_months: u32,
    pub applied_at: u64,
}

// Zero-Knowledge Privacy Claims types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Nullifier {
    pub hash: BytesN<32>, // 256-bit hash
}

#[contracttype]
#[derive(Clone)]
pub struct Commitment {
    pub hash: BytesN<32>, // 256-bit hash
    pub created_at: u64,
    pub vesting_id: u32,
    pub amount: i128,
    pub is_used: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct ZKClaimProof {
    pub commitment_hash: BytesN<32>,
    pub nullifier_hash: BytesN<32>,
    pub merkle_root: BytesN<32>,
    pub proof_data: Bytes, // Placeholder for actual ZK-SNARK proof
}

#[contracttype]
#[derive(Clone)]
pub struct PrivacyClaimEvent {
    pub nullifier: Nullifier,
    pub amount: i128,
    pub timestamp: u64,
    pub vesting_id: u32,
    pub is_private: bool,
}

#[contractevent]
#[derive(Clone)]
pub struct CommitmentCreated {
    #[topic]
    pub commitment_hash: BytesN<32>,
    #[topic]
    pub vesting_id: u32,
    pub amount: i128,
    pub created_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct PrivateClaimExecuted {
    #[topic]
    pub nullifier_hash: BytesN<32>,
    pub amount: i128,
    pub timestamp: u64,
}

// Stellar Horizon Path Payment Claim types
#[contracttype]
#[derive(Clone)]
pub struct PathPaymentConfig {
    pub destination_asset: Address, // USDC or other stablecoin
    pub min_destination_amount: i128,
    pub path: Vec<Address>, // Path of assets for the swap
    pub enabled: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct PathPaymentClaimEvent {
    pub beneficiary: Address,
    pub source_amount: i128,
    pub destination_amount: i128,
    pub destination_asset: Address,
    pub timestamp: u64,
    pub vesting_id: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct PathPaymentSimulation {
    pub source_amount: i128,
    pub estimated_destination_amount: i128,
    pub min_destination_amount: i128,
    pub path: Vec<Address>,
    pub can_execute: bool,
    pub reason: String,
    pub estimated_gas_fee: u64,
}
#[contractevent]
#[derive(Clone)]
pub struct PathPaymentConfigured {
    pub destination_asset: Address,
    pub min_destination_amount: i128,
    pub path: Vec<Address>,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct PathPaymentDisabled {
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct PathPaymentClaimExecuted {
    #[topic]
    pub user: Address,
    pub source_amount: i128,
    pub destination_amount: i128,
    pub destination_asset: Address,
    pub timestamp: u64,
    #[topic]
    pub vesting_id: u32,
}

// Lock-up period types
#[contracttype]
#[derive(Clone)]
pub struct LockupConfig {
    pub vesting_id: u32,
    pub lockup_duration_seconds: u64,
    pub enabled: bool,
    pub lockup_token_address: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct LockupConfigured {
    #[topic]
    pub vesting_id: u32,
    pub lockup_duration_seconds: u64,
    pub lockup_token_address: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct LockupDisabled {
    #[topic]
    pub vesting_id: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct LockupClaimExecuted {
    #[topic]
    pub user: Address,
    #[topic]
    pub vesting_id: u32,
    pub amount: i128,
    pub lockup_token_address: Address,
    pub unlock_time: u64,
    pub timestamp: u64,
}

// Beneficiary reassignment types (Issue 114)
#[contracttype]
#[derive(Clone)]
pub struct BeneficiaryReassignment {
    pub vesting_id: u32,
    pub current_beneficiary: Address,
    pub new_beneficiary: Address,
    pub requested_at: u64,
    pub effective_at: u64,
    pub total_amount: i128,
    pub requires_governance_veto: bool,
    pub is_executed: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct GovernanceVeto {
    pub reassignment_id: u32,
    pub veto_by: Address,
    pub veto_at: u64,
    pub reason: String,
    pub voting_power: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct VetoVote {
    pub voter: Address,
    pub reassignment_id: u32,
    pub vote_for_veto: bool,
    pub voting_power: i128,
    pub voted_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TokenSupplyInfo {
    pub total_supply: i128,
    pub last_updated: u64,
}

// Governance veto events
#[contractevent]
#[derive(Clone)]
pub struct BeneficiaryReassignmentRequested {
    #[topic]
    pub reassignment_id: u32,
    #[topic]
    pub vesting_id: u32,
    #[topic]
    pub current_beneficiary: Address,
    #[topic]
    pub new_beneficiary: Address,
    pub total_amount: i128,
    pub effective_at: u64,
    pub requires_governance_veto: bool,
}

#[contractevent]
#[derive(Clone)]
pub struct BeneficiaryReassignmentExecuted {
    #[topic]
    pub reassignment_id: u32,
    #[topic]
    pub vesting_id: u32,
    #[topic]
    pub old_beneficiary: Address,
    #[topic]
    pub new_beneficiary: Address,
    pub executed_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct VetoPeriodStarted {
    #[topic]
    pub reassignment_id: u32,
    #[topic]
    pub vesting_id: u32,
    pub veto_deadline: u64,
    pub threshold_percentage: u32,
}

#[contractevent]
#[derive(Clone)]
pub struct VetoVoteCast {
    #[topic]
    pub voter: Address,
    #[topic]
    pub reassignment_id: u32,
    pub vote_for_veto: bool,
    pub voting_power: i128,
    pub voted_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct ReassignmentVetoed {
    #[topic]
    pub reassignment_id: u32,
    #[topic]
    pub veto_triggered_by: Address,
    pub veto_power: i128,
    pub vetoed_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct ReassignmentApproved {
    #[topic]
    pub reassignment_id: u32,
    #[topic]
    pub approved_at: u64,
    pub total_veto_power: i128,
}

// LST Deposit support
#[contracttype]
#[derive(Clone)]
pub struct LSTConfig {
    pub vesting_id: u32,
    pub enabled: bool,
    pub lst_token_address: Address,
    pub base_token_address: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct LSTConfigured {
    #[topic]
    pub vesting_id: u32,
    pub lst_token_address: Address,
    pub base_token_address: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct LSTClaimExecuted {
    #[topic]
    pub user: Address,
    #[topic]
    pub vesting_id: u32,
    pub base_amount: i128,
    pub lst_amount: i128,
    pub lst_token_address: Address,
    pub timestamp: u64,
}

// ========== ISSUE #224: Global Reentrancy Guard ==========
// (no contracttype needed — stored as bool)

// ========== ISSUE #227: Maximum TVL Cap ==========
#[contracttype]
#[derive(Clone)]
pub struct TvlCapConfig {
    pub max_protocol_tvl: i128,
    pub current_tvl: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct TvlCapConfigured {
    pub max_protocol_tvl: i128,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct ScheduleCreated {
    #[topic]
    pub beneficiary: Address,
    #[topic]
    pub vesting_id: u32,
    pub amount: i128,
    pub new_tvl: i128,
    pub timestamp: u64,
}

// ========== ISSUE #229: Daily Withdrawal Rate Limit ==========
#[contracttype]
#[derive(Clone)]
pub struct DailyClaimRecord {
    pub beneficiary: Address,
    pub day_timestamp: u64, // truncated to day boundary
    pub claimed_today: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct RateLimitConfig {
    pub max_claim_per_day: i128,
    pub enabled: bool,
}

#[contractevent]
#[derive(Clone)]
pub struct RateLimitConfigured {
    pub max_claim_per_day: i128,
    pub timestamp: u64,
}

// ========== ISSUE #222: Yield-Harvesting Batch Relayer ==========
#[contracttype]
#[derive(Clone)]
pub struct HarvestAllResult {
    pub total_harvested: i128,
    pub relayer_reward: i128,
    pub vaults_processed: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct RelayerConfig {
    pub reward_bps: u32, // basis points (e.g. 50 = 0.5%)
    pub enabled: bool,
}

#[contractevent]
#[derive(Clone)]
pub struct HarvestAllExecuted {
    #[topic]
    pub relayer: Address,
    pub total_harvested: i128,
    pub relayer_reward: i128,
    pub vaults_processed: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct RelayerConfigured {
    pub reward_bps: u32,
    pub timestamp: u64,
}

// ========== ISSUE #205: Tax Withholding ==========
#[contracttype]
#[derive(Clone)]
pub struct TaxWithholdingConfig {
    pub tax_treasury_address: Address,
    pub tax_withholding_bps: u32,
    pub enabled: bool,
}

#[contractevent]
#[derive(Clone)]
pub struct TaxWithholdingConfigured {
    pub tax_treasury_address: Address,
    pub tax_withholding_bps: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct TaxWithholdingDisabled {
    pub timestamp: u64,
}

// ========== ISSUE #204: SEP-12 KYC ==========
#[contracttype]
#[derive(Clone)]
pub struct SEP12IdentityOracle {
    pub contract_address: Address,
    pub enabled: bool,
}

#[contractevent]
#[derive(Clone)]
pub struct SEP12OracleConfigured {
    pub oracle_address: Address,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct SEP12KYCDisabled {
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct KYCCheckFailed {
    #[topic]
    pub beneficiary: Address,
    pub reason: String,
    pub timestamp: u64,
}

// ========== ISSUE #203: Token Metadata ==========
#[contracttype]
#[derive(Clone)]
pub struct TokenMetadata {
    pub decimals: u32,
    pub asset_address: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct TokenMetadataRegistered {
    pub asset_address: Address,
    pub decimals: u32,
    pub timestamp: u64,
}

// ========== ISSUE #202: Vesting Grant ==========
#[contracttype]
#[derive(Clone)]
pub struct VestingGrant {
    pub vesting_id: u32,
    pub beneficiary: Address,
    pub created_at: u64,
    pub is_revocable: bool,
    pub revocability_expires_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct VestingGrantCreated {
    #[topic]
    pub vesting_id: u32,
    pub beneficiary: Address,
    pub is_revocable: bool,
    pub revocability_expires_at: u64,
    pub created_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct RevocabilityExpired {
    #[topic]
    pub vesting_id: u32,
    pub beneficiary: Address,
    pub expired_at: u64,
}
