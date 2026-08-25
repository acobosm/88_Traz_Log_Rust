use anchor_lang::prelude::*;
use crate::constants::{SEED_GLOBAL, SEED_PERSONNEL, SEED_EQUIPMENT, SEED_LOG};
use crate::state::{GlobalState, PersonnelAccount, EquipmentAccount, EquipmentStatus, ReportedCondition, Role, LogEntry};
use crate::error::TrazLogError;

pub fn handler(
    ctx: Context<LogMilestone>,
    _equipment_code: [u8; 32],
    notes: String,
    condition: ReportedCondition,
) -> Result<()> {
    require!(!ctx.accounts.global_state.is_paused, TrazLogError::SystemPaused);

    let equipment = &mut ctx.accounts.equipment;
    let log_entry  = &mut ctx.accounts.log_entry;

    log_entry.equipment_code = equipment.code;
    log_entry.notes          = notes;
    log_entry.condition      = condition.clone();
    log_entry.operator       = ctx.accounts.signer.key();
    log_entry.timestamp      = Clock::get()?.unix_timestamp;
    log_entry.entry_index    = equipment.log_count;
    log_entry.bump           = ctx.bumps.log_entry;

    equipment.reported_condition = condition;
    equipment.log_count += 1;

    Ok(())
}

#[derive(Accounts)]
#[instruction(equipment_code: [u8; 32])]
pub struct LogMilestone<'info> {
    #[account(mut)]
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

    // log_count como parte de la seed: cada reporte crea un PDA nuevo en vez de
    // sobrescribir uno solo, así queda el historial completo consultable on-chain
    #[account(
        init,
        payer = signer,
        space = 8 + LogEntry::INIT_SPACE,
        seeds = [SEED_LOG, equipment_code.as_ref(), &equipment.log_count.to_le_bytes()],
        bump,
    )]
    pub log_entry: Account<'info, LogEntry>,

    pub system_program: Program<'info, System>,
}
