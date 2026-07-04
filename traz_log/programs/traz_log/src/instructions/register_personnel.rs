use anchor_lang::prelude::*;
use crate::constants::{SEED_GLOBAL, SEED_PERSONNEL};
use crate::state::{GlobalState, PersonnelAccount, Role};
use crate::error::TrazLogError;
use crate::events::PersonnelRegistered;

// Phase 0: admin-only. Phase 3 will add OperationalBase path.
pub fn handler(
    ctx: Context<RegisterPersonnel>,
    name: String,
    specialty: String,
    role: Role,
) -> Result<()> {
    require!(!ctx.accounts.global_state.is_paused, TrazLogError::SystemPaused);

    let p = &mut ctx.accounts.new_personnel;
    p.wallet = ctx.accounts.wallet.key();
    p.name = name;
    p.specialty = specialty;
    p.is_active = true;
    p.role = role.clone();
    p.bump = ctx.bumps.new_personnel;

    emit!(PersonnelRegistered { wallet: p.wallet, role });
    Ok(())
}

#[derive(Accounts)]
pub struct RegisterPersonnel<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [SEED_GLOBAL],
        bump = global_state.bump,
        constraint = global_state.admin == signer.key() @ TrazLogError::Unauthorized,
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        payer = signer,
        space = 8 + PersonnelAccount::INIT_SPACE,
        seeds = [SEED_PERSONNEL, wallet.key().as_ref()],
        bump,
    )]
    pub new_personnel: Account<'info, PersonnelAccount>,

    /// The wallet address being registered as personnel
    pub wallet: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
