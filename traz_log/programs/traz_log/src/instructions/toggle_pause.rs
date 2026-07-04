use anchor_lang::prelude::*;
use crate::constants::SEED_GLOBAL;
use crate::state::GlobalState;
use crate::error::TrazLogError;

pub fn handler(ctx: Context<TogglePause>) -> Result<()> {
    ctx.accounts.global_state.is_paused = !ctx.accounts.global_state.is_paused;
    Ok(())
}

#[derive(Accounts)]
pub struct TogglePause<'info> {
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_GLOBAL],
        bump = global_state.bump,
        constraint = global_state.admin == signer.key() @ TrazLogError::Unauthorized,
    )]
    pub global_state: Account<'info, GlobalState>,
}
