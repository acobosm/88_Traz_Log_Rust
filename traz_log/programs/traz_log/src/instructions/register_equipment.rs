use anchor_lang::prelude::*;
use crate::constants::{SEED_GLOBAL, SEED_PERSONNEL, SEED_EQUIPMENT};
use crate::state::{GlobalState, PersonnelAccount, EquipmentAccount, EquipmentStatus, ReportedCondition, Role};
use crate::error::TrazLogError;
use crate::events::EquipmentRegistered;

pub fn handler(
    ctx: Context<RegisterEquipment>,
    code: [u8; 32],
    description: String,
    nominal_consumption: u64,
) -> Result<()> {
    require!(!ctx.accounts.global_state.is_paused, TrazLogError::SystemPaused);

    let e = &mut ctx.accounts.equipment;
    e.code = code;
    e.description = description;
    e.nominal_consumption = nominal_consumption;
    e.status = EquipmentStatus::Available;
    e.reported_condition = ReportedCondition::Operational;
    e.custodian = Pubkey::default();
    e.incident_id = 0;
    e.use_start_time = 0;
    e.bump = ctx.bumps.equipment;

    emit!(EquipmentRegistered { code });
    Ok(())
}

#[derive(Accounts)]
#[instruction(code: [u8; 32])]
pub struct RegisterEquipment<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(seeds = [SEED_GLOBAL], bump = global_state.bump)]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        seeds = [SEED_PERSONNEL, signer.key().as_ref()],
        bump = signer_personnel.bump,
        constraint = signer_personnel.role == Role::OperationalBase @ TrazLogError::Unauthorized,
        constraint = signer_personnel.is_active @ TrazLogError::InactivePersonnel,
    )]
    pub signer_personnel: Account<'info, PersonnelAccount>,

    #[account(
        init,
        payer = signer,
        space = 8 + EquipmentAccount::INIT_SPACE,
        seeds = [SEED_EQUIPMENT, code.as_ref()],
        bump,
    )]
    pub equipment: Account<'info, EquipmentAccount>,

    pub system_program: Program<'info, System>,
}
