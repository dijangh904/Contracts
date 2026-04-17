/// Example demonstrating diversified vesting functionality
/// 
/// This example shows how the new diversified vesting system works:
/// 1. Create a basket of multiple assets (e.g., 50% ProjectToken, 25% XLM, 25% USDC)
/// 2. Vest all assets simultaneously according to the same schedule
/// 3. Claim all assets proportionally as they vest

#[cfg(any(test, feature = "testutils"))]
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone)]
pub struct AssetAllocation {
    pub asset_id: Address,
    pub total_amount: i128,
    pub released_amount: i128,
    pub locked_amount: i128,
    pub percentage: u32, // Percentage in basis points (10000 = 100%)
}

#[contracttype]
#[derive(Clone)]
pub struct DiversifiedVault {
    pub allocations: Vec<AssetAllocation>,
    pub owner: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub creation_time: u64,
    pub is_initialized: bool,
}

/// Example: Create a diversified vesting schedule
/// 50% Project Token, 25% XLM, 25% USDC
pub fn create_example_diversified_vault(env: &Env) -> DiversifiedVault {
    // Generate dummy addresses for example purposes
    // NOTE: In production, these would be actual contract addresses
    #[cfg(any(test, feature = "testutils"))]
    let owner = Address::generate(env);
    #[cfg(not(any(test, feature = "testutils")))]
    let owner = Address::from_string_bytes(&soroban_sdk::Bytes::new(env));
    
    #[cfg(any(test, feature = "testutils"))]
    let project_token = Address::generate(env);
    #[cfg(not(any(test, feature = "testutils")))]
    let project_token = Address::from_string_bytes(&soroban_sdk::Bytes::new(env));
    
    #[cfg(any(test, feature = "testutils"))]
    let xlm_token = Address::generate(env);
    #[cfg(not(any(test, feature = "testutils")))]
    let xlm_token = Address::from_string_bytes(&soroban_sdk::Bytes::new(env));
    
    #[cfg(any(test, feature = "testutils"))]
    let usdc_token = Address::generate(env);
    #[cfg(not(any(test, feature = "testutils")))]
    let usdc_token = Address::from_string_bytes(&soroban_sdk::Bytes::new(env));
    
    // Create asset basket
    let mut asset_basket = Vec::new(env);
    
    // 50% Project Token (10,000 tokens)
    asset_basket.push_back(AssetAllocation {
        asset_id: project_token,
        total_amount: 10_000_0000000,
        released_amount: 0,
        locked_amount: 0,
        percentage: 5000,
    });
    
    // 25% XLM (5,000 XLM)
    asset_basket.push_back(AssetAllocation {
        asset_id: xlm_token,
        total_amount: 5_000_0000000,
        released_amount: 0,
        locked_amount: 0,
        percentage: 2500,
    });
    
    // 25% USDC (5,000 USDC)
    asset_basket.push_back(AssetAllocation {
        asset_id: usdc_token,
        total_amount: 5_000_0000000,
        released_amount: 0,
        locked_amount: 0,
        percentage: 2500,
    });
    
    let start_time = env.ledger().timestamp();
    let end_time = start_time + (4 * 365 * 24 * 60 * 60);
    
    DiversifiedVault {
        allocations: asset_basket,
        owner,
        start_time,
        end_time,
        creation_time: start_time,
        is_initialized: true,
    }
}

/// Calculate how much of each asset is claimable at current time
pub fn calculate_claimable_amounts(env: &Env, vault: &DiversifiedVault) -> Vec<(Address, i128)> {
    let mut claimable = Vec::new(env);
    let now = env.ledger().timestamp();
    
    if now <= vault.start_time {
        return claimable;
    }
    
    let total_duration = vault.end_time - vault.start_time;
    let elapsed = if now >= vault.end_time {
        total_duration
    } else {
        now - vault.start_time
    };
    
    for allocation in vault.allocations.iter() {
        let vested_amount = (allocation.total_amount * elapsed as i128) / total_duration as i128;
        let claimable_amount = vested_amount - allocation.released_amount;
        
        if claimable_amount > 0 {
            claimable.push_back((allocation.asset_id.clone(), claimable_amount));
        }
    }
    
    claimable
}

pub fn main() {
    // This function is required for the file to be compiled as an example binary
    let env = Env::default();
    let vault = create_example_diversified_vault(&env);
    let _ = calculate_claimable_amounts(&env, &vault);
}

// Key benefits of diversified vesting:
// 1. Risk Reduction: No exposure to single token's volatility.
// 2. Stable Value: XLM and USDC portions provide stability.
// 3. Attractive Compensation: Senior developers get stable assets.
// 4. Flexible Composition: Customizable per beneficiary.
// 5. Simultaneous Vesting: Maintains allocation percentages over time.