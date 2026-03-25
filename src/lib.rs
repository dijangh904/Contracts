#![no_std]
pub fn claim(env: Env, beneficiary: Address) -> Result<(), Error> {
    beneficiary.require_auth();

    let total_liabilities = get_total_locked(&env);
    let current_balance = get_contract_asset_balance(&env);

    // The Deficit Handler
    if current_balance < total_liabilities {
        // Emit the event for indexers and frontend alerts
        env.events().publish(
            (symbol_short!("Clawback"),),
            (current_balance, total_liabilities)
        );
        
        // Enter Safety Pause (Governance state)
        set_pause_state(&env, true);
        
        return Err(Error::DeficitDetected);
    }

    // ... proceed with normal vesting/claim logic if solvent
}