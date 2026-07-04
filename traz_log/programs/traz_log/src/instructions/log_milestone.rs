use anchor_lang::prelude::*;
use crate::constants::{SEED_GLOBAL, SEED_PERSONNEL, SEED_EQUIPMENT};
use crate::state::{GlobalState, PersonnelAccount, EquipmentAccount, EquipmentStatus, ReportedCondition, Role};
use crate::error::TrazLogError;

pub fn handler(
    ctx: Context<LogMilestone>,
    _equipment_code: [u8; 32],
    _notes: String,
    condition: ReportedCondition,
) -> Result<()> {
    require!(!ctx.accounts.global_state.is_paused, TrazLogError::SystemPaused);
    ctx.accounts.equipment.reported_condition = condition;
    Ok(())
}

#[derive(Accounts)]
#[instruction(equipment_code: [u8; 32])]
pub struct LogMilestone<'info> {
    pub signer: Signer<'info>,

    #[account(seeds = [SEED_GLOBAL], bump = global_state.bump)]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        seeds = [SEED_PERSONNEL, signer.key().as_ref()],
        bump = signer_personnel.bump,
        constraint = signer_personnel.role == Role::Operator @ TrazLogError::Unauthorized,
        constraint = signer_personnel.is_active @ TrazLogError::InactivePersonnel,
    )]
    pub signer_personnel: Account<'info, PersonnelAccount>,

    #[account(
        mut,
        seeds = [SEED_EQUIPMENT, equipment_code.as_ref()],
        bump = equipment.bump,
        constraint = equipment.custodian == signer.key() @ TrazLogError::NotCustodian,
        constraint = equipment.status == EquipmentStatus::InUse @ TrazLogError::EquipmentNotInUse,
    )]
    pub equipment: Account<'info, EquipmentAccount>,
}
