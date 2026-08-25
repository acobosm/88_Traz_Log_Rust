use anchor_lang::prelude::*;
use crate::constants::{SEED_GLOBAL, SEED_PERSONNEL, SEED_INCIDENT};
use crate::state::{GlobalState, PersonnelAccount, IncidentAccount, Role};
use crate::error::TrazLogError;
use crate::events::IncidentClosed;

pub fn handler(
    ctx: Context<CloseIncident>,
    _incident_id: u64,
) -> Result<()> {
    require!(!ctx.accounts.global_state.is_paused, TrazLogError::SystemPaused);
    let incident = &mut ctx.accounts.incident;
    // transición perezosa — no libera el equipo, cada operador lo hace con initiate_return
    incident.is_active = false;
    let id = incident.incident_id;
    emit!(IncidentClosed { incident_id: id });
    Ok(())
}

#[derive(Accounts)]
#[instruction(incident_id: u64)]
pub struct CloseIncident<'info> {
    pub signer: Signer<'info>,

    #[account(seeds = [SEED_GLOBAL], bump = global_state.bump)]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        seeds = [SEED_PERSONNEL, signer.key().as_ref()],
        bump = signer_personnel.bump,
        constraint = signer_personnel.role == Role::SceneCommander @ TrazLogError::Unauthorized,
        constraint = signer_personnel.is_active @ TrazLogError::InactivePersonnel,
    )]
    pub signer_personnel: Account<'info, PersonnelAccount>,

    #[account(
        mut,
        seeds = [SEED_INCIDENT, &incident_id.to_le_bytes()],
        bump = incident.bump,
        constraint = incident.is_active @ TrazLogError::IncidentAlreadyClosed,
    )]
    pub incident: Account<'info, IncidentAccount>,
}
