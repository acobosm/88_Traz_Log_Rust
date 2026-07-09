use anchor_lang::prelude::*;

#[error_code]
pub enum TrazLogError {
    #[msg("Unauthorized: insufficient role for this action")]
    Unauthorized,
    #[msg("System is currently paused")]
    SystemPaused,
    #[msg("Personnel account is inactive")]
    InactivePersonnel,
    #[msg("Equipment is not available for assignment")]
    EquipmentNotAvailable,
    #[msg("Equipment is not in use")]
    EquipmentNotInUse,
    #[msg("Caller is not the equipment custodian")]
    NotCustodian,
    #[msg("Incident is not active")]
    IncidentNotActive,
    #[msg("Incident is already closed")]
    IncidentAlreadyClosed,
    #[msg("Assigned operator must have the Operator role")]
    InvalidOperatorRole,
    #[msg("Invalid equipment status for this operation")]
    InvalidEquipmentStatus,
    #[msg("Risk level must be between 1 and 5")]
    InvalidRiskLevel,
    #[msg("Provided incident ID does not match current counter")]
    InvalidIncidentId,
    #[msg("Operator is already assigned to a different active incident")]
    OperatorAlreadyAssigned,
    #[msg("Only the commander who opened this incident can assign equipment to it")]
    NotIncidentCommander,
}
