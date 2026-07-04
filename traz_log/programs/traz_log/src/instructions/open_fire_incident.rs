use anchor_lang::prelude::*;
use crate::constants::{SEED_GLOBAL, SEED_PERSONNEL, SEED_INCIDENT};
use crate::state::{GlobalState, PersonnelAccount, IncidentAccount, Role};
use crate::error::TrazLogError;
use crate::events::IncidentOpened;

pub fn handler(
    ctx: Context<OpenFireIncident>,
    incident_id: u64,
    coordinates: String,
    risk_level: u8,
) -> Result<()> {
    require!(!ctx.accounts.global_state.is_paused, TrazLogError::SystemPaused);
    require!(risk_level >= 1 && risk_level <= 5, TrazLogError::InvalidRiskLevel);
    require!(
        incident_id == ctx.accounts.global_state.next_incident_id,
        TrazLogError::InvalidIncidentId
    );

    let clock = Clock::get()?;
    let incident = &mut ctx.accounts.incident;
    incident.incident_id = incident_id;
    incident.coordinates = coordinates;
    incident.risk_level = risk_level;
    incident.is_active = true;
    incident.opened_at = clock.unix_timestamp;
    incident.commander = ctx.accounts.signer.key();
    incident.bump = ctx.bumps.incident;

    let commander = incident.commander;
    ctx.accounts.global_state.next_incident_id += 1;

    emit!(IncidentOpened { incident_id, commander, risk_level });
    Ok(())
}

#[derive(Accounts)]
#[instruction(incident_id: u64)]
pub struct OpenFireIncident<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_GLOBAL],
        bump = global_state.bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        seeds = [SEED_PERSONNEL, signer.key().as_ref()],
        bump = signer_personnel.bump,
        constraint = signer_personnel.role == Role::SceneCommander @ TrazLogError::Unauthorized,
        constraint = signer_personnel.is_active @ TrazLogError::InactivePersonnel,
    )]
    pub signer_personnel: Account<'info, PersonnelAccount>,

    #[account(
        init,
        payer = signer,
        space = 8 + IncidentAccount::INIT_SPACE,
        seeds = [SEED_INCIDENT, &incident_id.to_le_bytes()],
        bump,
    )]
    pub incident: Account<'info, IncidentAccount>,

    pub system_program: Program<'info, System>,
}
