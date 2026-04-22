//! Example demonstrating Good Leaver / Bad Leaver termination functionality
//! 
//! This example shows how DAO administrators can use the terminate_schedule function
//! to handle employee departures with different termination states.

use soroban_sdk::{Env, Address, contracttype};
use vesting_vault::{VestingVaultClient, LeaverType};

pub fn main() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VestingVault);
    let client = VestingVaultClient::new(&env, &contract_id);

    // Setup addresses
    let dao_admin = Address::random(&env);
    let employee_alice = Address::random(&env); // Good leaver
    let employee_bob = Address::random(&env);   // Bad leaver  
    let dao_treasury = Address::random(&env);

    println!("=== Good Leaver / Bad Leaver Termination Example ===\n");

    // Scenario 1: Alice is leaving on good terms
    println!("Scenario 1: Alice (Good Leaver)");
    println!("Alice has been a great employee and is leaving amicably.");
    println!("She should keep her vested tokens but future vesting stops.");
    
    // In a real scenario, Alice might have:
    // - 60% of tokens vested (6,000 out of 10,000)
    // - 4,000 tokens still unvested
    // - 2,000 vested tokens already claimed
    
    client.terminate_schedule(
        &dao_admin,
        &employee_alice,
        &1u32, // vesting_id for Alice
        &LeaverType::GoodLeaver,
        &dao_treasury,
    );

    println!("Alice's vesting schedule terminated as Good Leaver:");
    println!("- Vested tokens (6,000): Retained for Alice to claim");
    println!("- Unvested tokens (4,000): Returned to treasury");
    println!("- Alice can still claim her 6,000 vested tokens\n");

    // Scenario 2: Bob is terminated for cause
    println!("Scenario 2: Bob (Bad Leaver)");
    println!("Bob violated company policy and is being terminated for cause.");
    println!("He should lose both unvested and unclaimed vested tokens.");
    
    // In a real scenario, Bob might have:
    // - 60% of tokens vested (6,000 out of 10,000)
    // - 4,000 tokens still unvested
    // - 2,000 vested tokens already claimed
    // - 4,000 vested tokens unclaimed
    
    client.terminate_schedule(
        &dao_admin,
        &employee_bob,
        &2u32, // vesting_id for Bob
        &LeaverType::BadLeaver,
        &dao_treasury,
    );

    println!("Bob's vesting schedule terminated as Bad Leaver:");
    println!("- Vested & claimed tokens (2,000): Already with Bob");
    println!("- Vested & unclaimed tokens (4,000): Forfeited to treasury");
    println!("- Unvested tokens (4,000): Returned to treasury");
    println!("- Bob keeps only his already claimed 2,000 tokens\n");

    // Verify termination status
    println!("Verification:");
    println!("Alice's schedule terminated: {}", 
             client.is_schedule_terminated_public(&1u32));
    println!("Bob's schedule terminated: {}", 
             client.is_schedule_terminated_public(&2u32));

    println!("\n=== Key Differences ===");
    println!("Good Leaver:");
    println!("- Keeps all vested tokens (claimed and unclaimed)");
    println!("- Loses only unvested tokens");
    println!("- Can still claim vested tokens in the future");
    
    println!("\nBad Leaver:");
    println!("- Keeps only already claimed vested tokens");
    println!("- Loses both unvested and unclaimed vested tokens");
    println!("- Cannot claim any additional tokens");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_scenarios() {
        main();
    }
}
