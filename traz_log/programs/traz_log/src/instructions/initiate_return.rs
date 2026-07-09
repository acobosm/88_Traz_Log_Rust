use anchor_lang::prelude::*;
use crate::constants::{SEED_GLOBAL, SEED_PERSONNEL, SEED_EQUIPMENT};
use crate::state::{GlobalState, PersonnelAccount, EquipmentAccount, EquipmentStatus, Role};
use crate::error::TrazLogError;

pub fn handler(
    ctx: Context<InitiateReturn>,
    _equipment_code: [u8; 32],
) -> Result<()> {
    require!(!ctx.accounts.global_state.is_paused, TrazLogError::SystemPaused);
    require!(
        ctx.accounts.equipment.status == EquipmentStatus::InUse,
        TrazLogError::EquipmentNotInUse
    );
    ctx.accounts.equipment.status = EquipmentStatus::Returning;

    let op = &mut ctx.accounts.signer_personnel;
    op.active_assignments = op.active_assignments.saturating_sub(1);
    if op.active_assignments == 0 {
        op.current_incident = None;
    }
    Ok(())
}

#[derive(Accounts)]
#[instruction(equipment_code: [u8; 32])]
pub struct InitiateReturn<'info> {
    pub signer: Signer<'info>,

    #[account(seeds = [SEED_GLOBAL], bump = global_state.bump)]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
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
    )]
    pub equipment: Account<'info, EquipmentAccount>,
}
