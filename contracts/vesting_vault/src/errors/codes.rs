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

    // 🔒 Reentrancy Guard (Issue #224)
    ReentrantCall = 500,

    // 📊 TVL Cap (Issue #227)
    TvlCapExceeded = 501,

    // ⏱️ Rate Limiting (Issue #229)
    DailyClaimLimitExceeded = 502,

    // 🌾 Harvest Relayer (Issue #222)
    RelayerNotEnabled = 503,
}