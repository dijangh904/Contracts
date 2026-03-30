#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Address, Env, String, Symbol, token, Vec};

pub mod receipt;
pub mod goal_escrow;

// =============================================
// DATA KEYS
// =============================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    VestingSchedule(u32),
    VestingScheduleCount,
    GroupReserve,

    // Gas Subsidy
    GasSubsidyTracker,
    GasTreasuryBalance,

    // #153 #100: Community Governance Veto on Final Claim
    FinalClaimVeto(u32),           // schedule_id -> bool (true = veto active)
    CommunityVoteThreshold,        // e.g. 66% of community votes required
}

// =============================================
// EXISTING STRUCTS (kept for context)
// =============================================

#[contracttype]
#[derive(Clone)]
pub struct GrantImpactMetadata {
    pub grant_id: u64,
    pub proposal_title: String,
    pub milestone_count: u32,
    pub impact_description: String,
    pub category: Option<String>,
    pub requested_by: Address,
    pub approved_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct VestingSchedule {
    pub id: u32,
    pub beneficiary: Address,
    pub total_amount: u128,
    pub asset: Address,
    pub start_time: u64,
    pub cliff_time: u64,
    pub vesting_duration: u64,
    pub released: u128,
    pub grant_impact: Option<GrantImpactMetadata>,
}

// NEW: Gas Subsidy Tracker
#[contracttype]
#[derive(Clone)]
pub struct GasSubsidyTracker {
    pub total_subsidized: u32,
    pub max_subsidies: u32,
    pub min_xlm_balance: u128,
}

// =============================================
// CONTRACT TRAIT
// =============================================

pub trait VestingVaultTrait {
    fn init(env: Env, admin: Address);

    fn create_vesting_schedule(...) -> u32;   // (your existing signature)

    fn claim(env: Env, beneficiary: Address, schedule_id: u32) -> u128;

    fn claim_with_subsidy(env: Env, beneficiary: Address, schedule_id: u32) -> u128;

    // NEW: Community Governance Veto on Final 10% Claim
    fn claim_final_with_community_approval(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
        community_votes_for: u32,      // Number of community votes in favor
        total_community_votes: u32     // Total votes cast
    ) -> u128;

    fn deposit_gas_treasury(env: Env, admin: Address, amount: u128);
    fn get_gas_subsidy_info(env: Env) -> GasSubsidyTracker;
    fn get_grant_impact(env: Env, schedule_id: u32) -> Option<GrantImpactMetadata>;
}

// =============================================
// CONTRACT IMPLEMENTATION
// =============================================

#[contract]
pub struct VestingVault;

#[contractimpl]
impl VestingVaultTrait for VestingVault {
    fn init(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::VestingScheduleCount, &0u32);

        let tracker = GasSubsidyTracker {
            total_subsidized: 0,
            max_subsidies: 100,
            min_xlm_balance: 5_0000000,
        };
        env.storage().instance().set(&DataKey::GasSubsidyTracker, &tracker);
        env.storage().instance().set(&DataKey::GasTreasuryBalance, &0u128);

        // Default community vote threshold = 66%
        env.storage().instance().set(&DataKey::CommunityVoteThreshold, &66u32);
    }

    // ... keep all your existing functions (create_vesting_schedule, claim, claim_with_subsidy, etc.) ...

    // NEW FUNCTION: Final Claim with Community Governance Veto
    fn claim_final_with_community_approval(
        env: Env,
        beneficiary: Address,
        schedule_id: u32,
        community_votes_for: u32,
        total_community_votes: u32
    ) -> u128 {
        beneficiary.require_auth();

        let mut schedule: VestingSchedule = env.storage()
            .instance()
            .get(&DataKey::VestingSchedule(schedule_id))
            .unwrap_or_else(|| panic!("Schedule not found"));

        if schedule.beneficiary != beneficiary {
            panic!("Not the beneficiary");
        }

        let current_time = env.ledger().timestamp();
        let total_vested = Self::calculate_vested_amount(&schedule, current_time);
        let already_released = schedule.released;

        let remaining = total_vested - already_released;
        if remaining == 0 {
            panic!("Nothing left to claim");
        }

        // Check if this is the final 10% claim
        let final_10_percent = schedule.total_amount / 10;
        let is_final_claim = remaining <= final_10_percent;

        if is_final_claim {
            // Community veto check
            let threshold: u32 = env.storage()
                .instance()
                .get(&DataKey::CommunityVoteThreshold)
                .unwrap_or(66);