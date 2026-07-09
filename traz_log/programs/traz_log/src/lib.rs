pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use events::*;
pub use instructions::*;
pub use state::*;

declare_id!("13p4xV6WHaPwWzno1F6Z6MeY9b9wLYtEMAnUF8ofdW75");

#[program]
pub mod traz_log {
    use super::*;

    /// Initializes GlobalState PDA. Called once by admin after deploying.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    /// Pauses or unpauses the system. Admin only.
    pub fn toggle_pause(ctx: Context<TogglePause>) -> Result<()> {
        toggle_pause::handler(ctx)
    }

    /// Registers a brigade member with an assigned role. Admin only (Phase 0).
    pub fn register_personnel(
        ctx: Context<RegisterPersonnel>,
        name: String,
        specialty: String,
        role: Role,
    ) -> Result<()> {
        register_personnel::handler(ctx, name, specialty, role)
    }

    /// Registers equipment in inventory. OperationalBase only.
    pub fn register_equipment(
        ctx: Context<RegisterEquipment>,
        code: [u8; 32],
        description: String,
        nominal_consumption: u64,
    ) -> Result<()> {
        register_equipment::handler(ctx, code, description, nominal_consumption)
    }

    /// Opens a new fire incident. SceneCommander only.
    /// incident_id must equal global_state.next_incident_id.
    pub fn open_fire_incident(
        ctx: Context<OpenFireIncident>,
        incident_id: u64,
        description: String,
        coordinates: String,
        risk_level: u8,
    ) -> Result<()> {
        open_fire_incident::handler(ctx, incident_id, description, coordinates, risk_level)
    }

    /// Assigns equipment to an operator within an active incident. SceneCommander only.
    pub fn assign_equipment(
        ctx: Context<AssignEquipment>,
        equipment_code: [u8; 32],
        incident_id: u64,
    ) -> Result<()> {
        assign_equipment::handler(ctx, equipment_code, incident_id)
    }

    /// Reports equipment condition from the field. Operator (custodian) only.
    pub fn log_milestone(
        ctx: Context<LogMilestone>,
        equipment_code: [u8; 32],
        notes: String,
        condition: ReportedCondition,
    ) -> Result<()> {
        log_milestone::handler(ctx, equipment_code, notes, condition)
    }

    /// Operator initiates equipment return. Equipment moves to Returning status.
    pub fn initiate_return(
        ctx: Context<InitiateReturn>,
        equipment_code: [u8; 32],
    ) -> Result<()> {
        initiate_return::handler(ctx, equipment_code)
    }

    /// Closes the incident (lazy model). SceneCommander only.
    pub fn close_incident(
        ctx: Context<CloseIncident>,
        incident_id: u64,
    ) -> Result<()> {
        close_incident::handler(ctx, incident_id)
    }
}
