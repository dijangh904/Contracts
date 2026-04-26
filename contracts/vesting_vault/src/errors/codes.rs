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

    // 🛡️ Security Council / Pause (500s)
    ContractPaused = 500,
    NotSecurityCouncilMember = 501,
    AlreadyPaused = 502,
    NotPaused = 503,

    // 🔄 Upgrade (600s)
    UpgradeProposalExists = 600,
    NoUpgradeProposal = 601,
    UpgradeTimelockNotElapsed = 602,
    UpgradeVoteThresholdNotMet = 603,
    AlreadyVotedOnUpgrade = 604,
    UpgradeAlreadyExecuted = 605,

    // 💸 Double-spend (700s)
    NothingLeftToClaim = 700,
}