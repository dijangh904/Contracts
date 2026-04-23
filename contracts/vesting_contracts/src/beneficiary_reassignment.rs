#![no_std]
use soroban_sdk::{
    contracttype,
    contracterror,
    Address,
    Env,
    Vec,
    String,
    Symbol,
};

/// Beneficiary reassignment errors
#[contracterror]
#[repr(u32)]
pub enum ReassignmentError {
    InvalidVaultId = 1,
    VaultNotActive = 2,
    InvalidNewBeneficiary = 3,
    ReassignmentAlreadyExists = 4,
    InsufficientApprovals = 5,
    ApprovalExpired = 6,
    UnauthorizedApprover = 7,
    ReassignmentCompleted = 8,
    SocialProofInvalid = 9,
    EmergencyRejection = 10,
}

/// Reassignment request status
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ReassignmentStatus {
    None,
    Pending(Vec<Address>), // List of required approvers
    Approved,             // All approvals received
    Rejected,             // Reassignment rejected
    Completed,            // Reassignment completed
}

/// Social recovery proof types
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum SocialProofType {
    DeathCertificate,      // Death certificate
    LostKeys,            // Lost private keys
    CourtOrder,          // Court order for reassignment
    MultiSig,            // Multi-signature from trusted parties
    EmergencyContact,      // Emergency contact verification
}

/// Beneficiary reassignment request
#[derive(Clone)]
#[contracttype]
pub struct ReassignmentRequest {
    pub vault_id: u64,
    pub current_beneficiary: Address,
    pub new_beneficiary: Address,
    pub requested_at: u64,
    pub expires_at: u64,
    pub social_proof_type: SocialProofType,
    pub social_proof_hash: [u8; 32], // Hash of social proof document
    pub social_proof_ipfs: String,   // IPFS CID of social proof
    pub reason: String,
    pub status: ReassignmentStatus,
    pub approvals: Vec<Address>,     // Received approvals
    pub required_approvals: u32,    // Required number of approvals
}

/// DAO admin council member
#[derive(Clone)]
#[contracttype]
pub struct DAOMember {
    pub address: Address,
    pub joined_at: u64,
    pub is_active: bool,
    pub role: String, // "admin", "council", "recovery"
}

/// Reassignment configuration
#[derive(Clone)]
#[contracttype]
pub struct ReassignmentConfig {
    pub required_approvals: u32,        // Default: 2/3 multi-sig
    pub approval_window: u64,           // Time to approve (default: 7 days)
    pub emergency_enabled: bool,         // Emergency reassignment enabled
    pub social_proof_required: bool,     // Social proof required
    pub max_reassignments_per_vault: u32, // Limit reassignments
}

/// Storage keys for beneficiary reassignment
pub const REASSIGNMENT_REQUESTS: Bytes = Bytes::from_short_bytes("REASSIGNMENT_REQUESTS");
pub const DAO_MEMBERS: Bytes = Bytes::from_short_bytes("DAO_MEMBERS");
pub const REASSIGNMENT_CONFIG: Bytes = Bytes::from_short_bytes("REASSIGNMENT_CONFIG");
pub const VAULT_REASSIGNMENTS: Bytes = Bytes::from_short_bytes("VAULT_REASSIGNMENTS");

/// Beneficiary Reassignment Manager
pub struct BeneficiaryReassignment;

impl BeneficiaryReassignment {
    /// Initialize DAO council and reassignment config
    pub fn initialize(
        env: &Env,
        admin: Address,
        initial_members: Vec<Address>,
        required_approvals: u32,
        approval_window: u64,
    ) -> Result<(), ReassignmentError> {
        // Verify admin authorization (in production, add proper admin check)
        
        // Initialize DAO members
        let mut members = Vec::new(env);
        let now = env.ledger().timestamp();
        
        for member in initial_members.iter() {
            let dao_member = DAOMember {
                address: member.clone(),
                joined_at: now,
                is_active: true,
                role: "council".to_string(),
            };
            members.push_back(dao_member);
        }
        
        env.storage().persistent().set(&DAO_MEMBERS, &members);
        
        // Initialize configuration
        let config = ReassignmentConfig {
            required_approvals,
            approval_window: approval_window,
            emergency_enabled: true,
            social_proof_required: true,
            max_reassignments_per_vault: 3,
        };
        
        env.storage().persistent().set(&REASSIGNMENT_CONFIG, &config);
        
        Ok(())
    }

    /// Create beneficiary reassignment request
    pub fn create_reassignment_request(
        env: &Env,
        current_beneficiary: Address,
        new_beneficiary: Address,
        vault_id: u64,
        social_proof_type: SocialProofType,
        social_proof_hash: [u8; 32],
        social_proof_ipfs: String,
        reason: String,
    ) -> Result<(), ReassignmentError> {
        current_beneficiary.require_auth();
        
        // Validate vault exists and is active
        if !Self::is_vault_active(env, vault_id) {
            return Err(ReassignmentError::VaultNotActive);
        }
        
        // Validate new beneficiary
        if new_beneficiary == current_beneficiary {
            return Err(ReassignmentError::InvalidNewBeneficiary);
        }
        
        // Check if reassignment already exists
        if Self::has_pending_reassignment(env, vault_id) {
            return Err(ReassignmentError::ReassignmentAlreadyExists);
        }
        
        // Check reassignment limit
        if Self::exceeds_reassignment_limit(env, vault_id) {
            return Err(ReassignmentError::InvalidVaultId);
        }
        
        let config = Self::get_reassignment_config(env);
        let now = env.ledger().timestamp();
        let expires_at = now + config.approval_window;
        
        // Get required approvers (active DAO council members)
        let required_approvers = Self::get_active_council_members(env);
        let required_count = config.required_approvals.min(required_approvers.len() as u32);
        
        let request = ReassignmentRequest {
            vault_id,
            current_beneficiary: current_beneficiary.clone(),
            new_beneficiary: new_beneficiary.clone(),
            requested_at: now,
            expires_at,
            social_proof_type: social_proof_type.clone(),
            social_proof_hash,
            social_proof_ipfs: social_proof_ipfs.clone(),
            reason: reason.clone(),
            status: ReassignmentStatus::Pending(required_approvers.clone()),
            approvals: Vec::new(env),
            required_approvals: required_count,
        };
        
        // Store request
        let request_key = (REASSIGNMENT_REQUESTS, vault_id);
        env.storage().persistent().set(&request_key, &request);
        
        // Increment vault reassignment count
        Self::increment_vault_reassignments(env, vault_id);
        
        // Emit event
        ReassignmentRequested {
            vault_id,
            current_beneficiary,
            new_beneficiary,
            social_proof_type,
            expires_at,
            reason,
        }.publish(env);
        
        Ok(())
    }

    /// Approve reassignment request (DAO council member)
    pub fn approve_reassignment(
        env: &Env,
        approver: Address,
        vault_id: u64,
    ) -> Result<(), ReassignmentError> {
        approver.require_auth();
        
        // Verify approver is active DAO council member
        if !Self::is_active_council_member(env, approver.clone()) {
            return Err(ReassignmentError::UnauthorizedApprover);
        }
        
        let request_key = (REASSIGNMENT_REQUESTS, vault_id);
        let mut request: ReassignmentRequest = env.storage().persistent()
            .get(&request_key)
            .ok_or(ReassignmentError::InvalidVaultId)?;
        
        // Check if request is still pending
        match &request.status {
            ReassignmentStatus::Pending(_) => {},
            _ => return Err(ReassignmentError::ReassignmentCompleted),
        }
        
        // Check if not expired
        if env.ledger().timestamp() > request.expires_at {
            return Err(ReassignmentError::ApprovalExpired);
        }
        
        // Check if already approved
        if request.approvals.iter().any(|addr| addr == &approver) {
            return Ok(()); // Already approved, no error
        }
        
        // Add approval
        request.approvals.push_back(approver);
        
        // Check if sufficient approvals received
        if request.approvals.len() >= request.required_approvals as usize {
            request.status = ReassignmentStatus::Approved;
        }
        
        // Update request
        env.storage().persistent().set(&request_key, &request);
        
        // Emit approval event
        ReassignmentApproved {
            vault_id,
            approver,
            approvals_received: request.approvals.len() as u32,
            required_approvals: request.required_approvals,
        }.publish(env);
        
        // If fully approved, complete reassignment
        if request.approvals.len() >= request.required_approvals as usize {
            Self::complete_reassignment(env, vault_id)?;
        }
        
        Ok(())
    }

    /// Complete beneficiary reassignment
    pub fn complete_reassignment(
        env: &Env,
        vault_id: u64,
    ) -> Result<(), ReassignmentError> {
        let request_key = (REASSIGNMENT_REQUESTS, vault_id);
        let request: ReassignmentRequest = env.storage().persistent()
            .get(&request_key)
            .ok_or(ReassignmentError::InvalidVaultId)?;
        
        // Mark as completed
        let mut completed_request = request.clone();
        completed_request.status = ReassignmentStatus::Completed;
        env.storage().persistent().set(&request_key, &completed_request);
        
        // Update vault owner (this would integrate with main vault logic)
        // In a full implementation, this would call the vault's transfer function
        
        // Emit completion event
        ReassignmentCompleted {
            vault_id,
            old_beneficiary: request.current_beneficiary,
            new_beneficiary: request.new_beneficiary,
            completed_at: env.ledger().timestamp(),
        }.publish(env);
        
        Ok(())
    }

    /// Emergency reassignment (bypasses normal approval process)
    pub fn emergency_reassignment(
        env: &Env,
        emergency_admin: Address,
        vault_id: u64,
        new_beneficiary: Address,
        emergency_reason: String,
        social_proof_type: SocialProofType,
        social_proof_hash: [u8; 32],
        social_proof_ipfs: String,
    ) -> Result<(), ReassignmentError> {
        emergency_admin.require_auth();
        
        // Verify emergency admin privileges
        if !Self::is_emergency_admin(env, emergency_admin.clone()) {
            return Err(ReassignmentError::UnauthorizedApprover);
        }
        
        // Check if emergency reassignment is enabled
        let config = Self::get_reassignment_config(env);
        if !config.emergency_enabled {
            return Err(ReassignmentError::EmergencyRejection);
        }
        
        // Validate vault and new beneficiary
        if !Self::is_vault_active(env, vault_id) {
            return Err(ReassignmentError::VaultNotActive);
        }
        
        // Create emergency reassignment request
        let request = ReassignmentRequest {
            vault_id,
            current_beneficiary: Address::from_array([0; 32]), // Will be filled from vault
            new_beneficiary: new_beneficiary.clone(),
            requested_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp(), // Immediate
            social_proof_type: social_proof_type.clone(),
            social_proof_hash,
            social_proof_ipfs: social_proof_ipfs.clone(),
            reason: emergency_reason.clone(),
            status: ReassignmentStatus::Approved, // Auto-approved
            approvals: Vec::from_array(env, &[emergency_admin]),
            required_approvals: 1,
        };
        
        // Store and complete immediately
        let request_key = (REASSIGNMENT_REQUESTS, vault_id);
        env.storage().persistent().set(&request_key, &request);
        
        // Complete reassignment
        Self::complete_reassignment(env, vault_id)?;
        
        // Emit emergency event
        EmergencyReassignment {
            vault_id,
            emergency_admin,
            new_beneficiary,
            social_proof_type,
            reason,
        }.publish(env);
        
        Ok(())
    }

    /// Get reassignment request status
    pub fn get_reassignment_status(env: &Env, vault_id: u64) -> Option<ReassignmentRequest> {
        let request_key = (REASSIGNMENT_REQUESTS, vault_id);
        env.storage().persistent().get(&request_key)
    }

    /// Get active DAO council members
    pub fn get_active_council_members(env: &Env) -> Vec<Address> {
        let members: Vec<DAOMember> = env.storage().persistent()
            .get(&DAO_MEMBERS)
            .unwrap_or_else(|| Vec::new(env));
        
        let mut active_members = Vec::new(env);
        for member in members.iter() {
            if member.is_active && member.role == "council" {
                active_members.push_back(member.address);
            }
        }
        
        active_members
    }

    /// Add DAO council member
    pub fn add_dao_member(
        env: &Env,
        admin: Address,
        member_address: Address,
        role: String,
    ) -> Result<(), ReassignmentError> {
        // Verify admin authorization
        if !Self::is_emergency_admin(env, admin) {
            return Err(ReassignmentError::UnauthorizedApprover);
        }
        
        let members_key = DAO_MEMBERS;
        let mut members: Vec<DAOMember> = env.storage().persistent()
            .get(&members_key)
            .unwrap_or_else(|| Vec::new(env));
        
        // Check if member already exists
        if members.iter().any(|m| m.address == member_address) {
            return Ok(()); // Already exists
        }
        
        let new_member = DAOMember {
            address: member_address,
            joined_at: env.ledger().timestamp(),
            is_active: true,
            role,
        };
        
        members.push_back(new_member);
        env.storage().persistent().set(&members_key, &members);
        
        Ok(())
    }

    // Private helper methods

    fn is_vault_active(env: &Env, vault_id: u64) -> bool {
        // This would integrate with the main vault system
        // For now, assume vault exists and is active
        true
    }

    fn has_pending_reassignment(env: &Env, vault_id: u64) -> bool {
        let request_key = (REASSIGNMENT_REQUESTS, vault_id);
        if let Some(request) = env.storage().persistent().get::<_, ReassignmentRequest>(&request_key) {
            matches!(request.status, ReassignmentStatus::Pending(_))
        } else {
            false
        }
    }

    fn exceeds_reassignment_limit(env: &Env, vault_id: u64) -> bool {
        let config = Self::get_reassignment_config(env);
        let count_key = (VAULT_REASSIGNMENTS, vault_id);
        let current_count: u32 = env.storage().persistent()
            .get(&count_key)
            .unwrap_or(0);
        
        current_count >= config.max_reassignments_per_vault
    }

    fn increment_vault_reassignments(env: &Env, vault_id: u64) {
        let count_key = (VAULT_REASSIGNMENTS, vault_id);
        let current_count: u32 = env.storage().persistent()
            .get(&count_key)
            .unwrap_or(0);
        
        env.storage().persistent().set(&count_key, &(current_count + 1));
    }

    fn get_reassignment_config(env: &Env) -> ReassignmentConfig {
        env.storage().persistent()
            .get(&REASSIGNMENT_CONFIG)
            .unwrap_or_else(|| {
                ReassignmentConfig {
                    required_approvals: 2,
                    approval_window: 7 * 24 * 60 * 60, // 7 days
                    emergency_enabled: true,
                    social_proof_required: true,
                    max_reassignments_per_vault: 3,
                }
            })
    }

    fn is_active_council_member(env: &Env, member: Address) -> bool {
        let members: Vec<DAOMember> = env.storage().persistent()
            .get(&DAO_MEMBERS)
            .unwrap_or_else(|| Vec::new(env));
        
        members.iter().any(|m| m.address == member && m.is_active && m.role == "council")
    }

    fn is_emergency_admin(env: &Env, admin: Address) -> bool {
        // This would integrate with the main admin system
        // For now, assume any address can be emergency admin
        true
    }
}

/// Events for beneficiary reassignment
#[contractevent]
pub struct ReassignmentRequested {
    #[topic]
    pub vault_id: u64,
    #[topic]
    pub current_beneficiary: Address,
    #[topic]
    pub new_beneficiary: Address,
    pub social_proof_type: SocialProofType,
    pub expires_at: u64,
    pub reason: String,
}

#[contractevent]
pub struct ReassignmentApproved {
    #[topic]
    pub vault_id: u64,
    #[topic]
    pub approver: Address,
    pub approvals_received: u32,
    pub required_approvals: u32,
}

#[contractevent]
pub struct ReassignmentCompleted {
    #[topic]
    pub vault_id: u64,
    #[topic]
    pub old_beneficiary: Address,
    #[topic]
    pub new_beneficiary: Address,
    pub completed_at: u64,
}

#[contractevent]
pub struct EmergencyReassignment {
    #[topic]
    pub vault_id: u64,
    #[topic]
    pub emergency_admin: Address,
    #[topic]
    pub new_beneficiary: Address,
    pub social_proof_type: SocialProofType,
    pub reason: String,
}

/// Public interface for beneficiary reassignment
pub trait BeneficiaryReassignmentTrait {
    /// Create reassignment request
    fn create_reassignment_request(
        env: Env,
        current_beneficiary: Address,
        new_beneficiary: Address,
        vault_id: u64,
        social_proof_type: SocialProofType,
        social_proof_hash: [u8; 32],
        social_proof_ipfs: String,
        reason: String,
    ) -> Result<(), ReassignmentError>;

    /// Approve reassignment request
    fn approve_reassignment(
        env: Env,
        approver: Address,
        vault_id: u64,
    ) -> Result<(), ReassignmentError>;

    /// Emergency reassignment
    fn emergency_reassignment(
        env: Env,
        emergency_admin: Address,
        vault_id: u64,
        new_beneficiary: Address,
        emergency_reason: String,
        social_proof_type: SocialProofType,
        social_proof_hash: [u8; 32],
        social_proof_ipfs: String,
    ) -> Result<(), ReassignmentError>;

    /// Get reassignment status
    fn get_reassignment_status(env: Env, vault_id: u64) -> Option<ReassignmentRequest>;

    /// Get active council members
    fn get_active_council_members(env: Env) -> Vec<Address>;

    /// Add DAO member
    fn add_dao_member(
        env: Env,
        admin: Address,
        member_address: Address,
        role: String,
    ) -> Result<(), ReassignmentError>;
}
