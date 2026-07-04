use anchor_lang::prelude::*;
use crate::constants::SEED_GLOBAL;
use crate::state::GlobalState;

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let state = &mut ctx.accounts.global_state;
    state.next_incident_id = 0;
    state.is_paused = false;
    state.admin = ctx.accounts.admin.key();
    state.bump = ctx.bumps.global_state;
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + GlobalState::INIT_SPACE,
        seeds = [SEED_GLOBAL],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
}
