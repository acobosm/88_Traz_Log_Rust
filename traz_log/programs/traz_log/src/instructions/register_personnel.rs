use anchor_lang::prelude::*;
use crate::constants::{SEED_GLOBAL, SEED_PERSONNEL};
use crate::state::{GlobalState, PersonnelAccount, Role};
use crate::error::TrazLogError;
use crate::events::PersonnelRegistered;

pub fn handler(
    ctx: Context<RegisterPersonnel>,
    name: String,
    specialty: String,
    role: Role,
) -> Result<()> {
    require!(!ctx.accounts.global_state.is_paused, TrazLogError::SystemPaused);

    let is_admin = ctx.accounts.global_state.admin == ctx.accounts.signer.key();
    let is_operational_base = {
        let info = ctx.accounts.signer_personnel.to_account_info();
        if info.owner == ctx.program_id {
            let data = info.try_borrow_data()?;
            data.len() > 8
                && PersonnelAccount::try_deserialize(&mut &data[..])
                    .map(|acc| acc.role == Role::OperationalBase && acc.is_active)
                    .unwrap_or(false)
        } else {
            false
        }
    };
    require!(is_admin || is_operational_base, TrazLogError::Unauthorized);

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
    )]
    pub global_state: Account<'info, GlobalState>,

    /// CHECK: PDA derivation verified via seeds. May be uninitialized (Admin
    /// calling without their own PersonnelAccount) — checked manually in the
    /// handler to allow Admin OR an active OperationalBase to register personnel.
    #[account(seeds = [SEED_PERSONNEL, signer.key().as_ref()], bump)]
    pub signer_personnel: UncheckedAccount<'info>,

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
