use anchor_lang::prelude::*;
use crate::constants::{SEED_GLOBAL, SEED_PERSONNEL, SEED_EQUIPMENT, SEED_INCIDENT};
use crate::state::{GlobalState, PersonnelAccount, EquipmentAccount, IncidentAccount, EquipmentStatus, Role};
use crate::error::TrazLogError;
use crate::events::EquipmentAssigned;

pub fn handler(
    ctx: Context<AssignEquipment>,
    equipment_code: [u8; 32],
    _incident_id: u64,
) -> Result<()> {
    require!(!ctx.accounts.global_state.is_paused, TrazLogError::SystemPaused);

    let clock = Clock::get()?;
    let e = &mut ctx.accounts.equipment;
    e.status = EquipmentStatus::InUse;
    e.custodian = ctx.accounts.operator_wallet.key();
    e.incident_id = ctx.accounts.incident.incident_id;
    e.use_start_time = clock.unix_timestamp;

    let incident_id = e.incident_id;
    let operator = e.custodian;

    emit!(EquipmentAssigned { incident_id, equipment_code, operator });
    Ok(())
}

#[derive(Accounts)]
#[instruction(equipment_code: [u8; 32], incident_id: u64)]
pub struct AssignEquipment<'info> {
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
        seeds = [SEED_EQUIPMENT, equipment_code.as_ref()],
        bump = equipment.bump,
        constraint = equipment.status == EquipmentStatus::Available @ TrazLogError::EquipmentNotAvailable,
    )]
    pub equipment: Account<'info, EquipmentAccount>,

    #[account(
        seeds = [SEED_INCIDENT, &incident_id.to_le_bytes()],
        bump = incident.bump,
        constraint = incident.is_active @ TrazLogError::IncidentNotActive,
    )]
    pub incident: Account<'info, IncidentAccount>,

    #[account(
        seeds = [SEED_PERSONNEL, operator_wallet.key().as_ref()],
        bump = operator_personnel.bump,
        constraint = operator_personnel.role == Role::Operator @ TrazLogError::InvalidOperatorRole,
        constraint = operator_personnel.is_active @ TrazLogError::InactivePersonnel,
    )]
    pub operator_personnel: Account<'info, PersonnelAccount>,

    pub operator_wallet: SystemAccount<'info>,
}
