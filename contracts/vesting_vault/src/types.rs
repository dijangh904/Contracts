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

// Tax Withholding types for Issue #205
#[contracttype]
#[derive(Clone)]
pub struct TaxWithholdingConfig {
    pub tax_treasury_address: Address, // DAO Tax Treasury address
    pub tax_withholding_bps: u32,      // Basis points (10000 = 100%)
    pub enabled: bool,
}

#[contractevent]
#[derive(Clone)]
pub struct TaxWithholdingExecuted {
    #[topic]
    pub beneficiary: Address,
    #[topic]
    pub gross_amount: i128,
    #[topic]
    pub tax_amount: i128,
    #[topic]
    pub net_amount: i128,
    #[topic]
    pub tax_treasury: Address,
    pub timestamp: u64,
    pub vesting_id: u32,
}

// SEP-12 KYC types for Issue #204
#[contracttype]
#[derive(Clone)]
pub struct SEP12IdentityOracle {
    pub contract_address: Address,
    pub enabled: bool,
}

#[contractevent]
#[derive(Clone)]
pub struct KYCCheckFailed {
    #[topic]
    pub beneficiary: Address,
    pub reason: String,
    pub timestamp: u64,
}

// Token Precision types for Issue #203
#[contracttype]
#[derive(Clone)]
pub struct TokenMetadata {
    pub decimals: u32,
    pub asset_address: Address,
}

// Revocability Expiration types for Issue #202
#[contracttype]
#[derive(Clone)]
pub struct VestingGrant {
    pub vesting_id: u32,
    pub beneficiary: Address,
    pub created_at: u64,
    pub is_revocable: bool,
    pub revocability_expires_at: u64, // 12 months after creation
}

#[contractevent]
#[derive(Clone)]
pub struct RevocabilityExpired {
    #[topic]
    pub vesting_id: u32,
    #[topic]
    pub beneficiary: Address,
    pub expired_at: u64,
}

// Additional event types for the four issues
#[contractevent]
#[derive(Clone)]
pub struct TaxWithholdingConfigured {
    #[topic]
    pub tax_treasury_address: Address,
    #[topic]
    pub tax_withholding_bps: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct TaxWithholdingDisabled {
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct SEP12OracleConfigured {
    #[topic]
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
pub struct TokenMetadataRegistered {
    #[topic]
    pub asset_address: Address,
    #[topic]
    pub decimals: u32,
    pub timestamp: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct VestingGrantCreated {
    #[topic]
    pub vesting_id: u32,
    #[topic]
    pub beneficiary: Address,
    #[topic]
    pub is_revocable: bool,
    pub revocability_expires_at: u64,
    pub created_at: u64,
}

#[contractevent]
#[derive(Clone)]
pub struct VestingGrantRevoked {
    #[topic]
    pub vesting_id: u32,
    #[topic]
    pub beneficiary: Address,
    pub reason: String,
    pub revoked_at: u64,
}

