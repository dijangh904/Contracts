use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    // 🔐 General (100s)
    Unauthorized = 100,
    InvalidInput = 101,

    // ⏳ Vesting (200s)
    VestingNotFound = 200,
    CliffNotReached = 205,
    NothingToClaim = 206,
    AlreadyFullyClaimed = 207,

    // 💰 Financial (300s)
    InsufficientBalance = 300,

    // 🧪 LST Auto-Compounding (310s)
    /// #154: LST not configured for this vesting schedule
    LSTNotConfigured = 310,
    /// #154: LST auto-compounding not enabled
    LSTNotEnabled = 311,
    /// #154: LST pool shares not initialized
    LSTPoolNotInitialized = 312,
    /// #154: User has no shares in the LST pool
    NoUserShares = 313,
    /// #154: No shares to unbond
    NoSharesToUnbond = 314,
    /// #154: Unbonding already pending for this user
    UnbondingAlreadyPending = 315,
    /// #154: Unbonding queue is full (rate limit)
    UnbondingQueueFull = 316,
    /// #154: Unbonding period has not elapsed yet
    UnbondingPeriodNotElapsed = 317,
    /// #154: No unbonding request found
    NoUnbondingRequest = 318,
    /// #154: Exchange rate manipulation suspected
    ExchangeRateManipulationSuspected = 319,

    // 📜 Compliance (400s)
    KycNotCompleted = 400,
    KycExpired = 401,
    AddressSanctioned = 402,
    JurisdictionRestricted = 403,
    LegalSignatureMissing = 404,
    LegalSignatureInvalid = 405,
    ComplianceCheckFailed = 406,
    AmlThresholdExceeded = 407,
    RiskRatingTooHigh = 408,
    DocumentVerificationFailed = 409,
    AccreditationStatusInvalid = 410,
    TaxComplianceFailed = 411,
    RegulatoryBlockActive = 412,
    WhitelistNotApproved = 413,
    BlacklistViolation = 414,
    GeofencingRestriction = 415,
    IdentityVerificationExpired = 416,
    SourceOfFundsNotVerified = 417,
    BeneficialOwnerNotVerified = 418,
    PoliticallyExposedPerson = 419,
    SanctionsListHit = 420,

    // ⚙️ System (900s)
    Overflow = 900,

    // 🗳️ Governance / DAO (500s)
    /// #223: No unvested balance found for the queried address
    NoUnvestedBalance = 500,

    // 🔑 Admin Recovery (600s)
    /// #226: Admin dead-man's switch not configured
    AdminSwitchNotConfigured = 600,
    /// #226: Admin inactivity timeout has not elapsed yet
    AdminInactivityNotElapsed = 601,
    /// #226: Admin switch already triggered
    AdminSwitchAlreadyTriggered = 602,
    /// #226: Recovery address cannot be the same as admin
    RecoveryAddressInvalid = 603,

    // 🔮 Oracle (700s)
    /// #228: Oracle circuit breaker is currently tripped — vault is frozen
    OracleCircuitBreakerActive = 700,
    /// #228: Price deviation exceeds the 30% threshold
    OraclePriceDeviationTooHigh = 701,

    // 🛡️ Self-Destruct Prevention (800s)
    /// #231: Cannot upgrade/delete contract while unvested balance > 0
    UpgradeBlockedByUnvestedFunds = 800,
}