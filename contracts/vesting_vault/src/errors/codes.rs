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

    // 🌉 Cross-Chain Bridge (1000s)
    /// #268: Wormhole VAA signature verification failed
    InvalidBridgeSignature = 1000,
    /// #268: Bridge is currently paused
    BridgePaused = 1001,
    /// #268: Destination chain is not supported
    UnsupportedChain = 1002,
    /// #268: Bridge amount exceeds maximum limit
    BridgeAmountExceedsLimit = 1003,
    /// #268: Bridge cooldown period has not elapsed
    BridgeCooldownNotElapsed = 1004,
    /// #268: Replay attack detected - nonce already used
    NonceAlreadyUsed = 1005,
    /// #268: VAA payload is malformed or invalid
    InvalidVaaPayload = 1006,
    /// #268: VAA sequence number is invalid or out of order
    InvalidVaaSequence = 1007,
    /// #268: Destination address cannot be altered during transit
    DestinationAddressMismatch = 1008,
    /// #268: Bridge not configured
    BridgeNotConfigured = 1009,
}