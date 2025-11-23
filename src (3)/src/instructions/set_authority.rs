use {
    crate::{constants::*, errors::Errors, state::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Address validated using constraint
    #[account()]
    pub new_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_SEED.as_ref()],
        bump,
        constraint = global.initialized == true @ Errors::NotInitialized,
        constraint = global.authority == payer.key() @ Errors::NotAuthorized,
    )]
    pub global: Box<Account<'info, Global>>,

    pub system_program: Program<'info, System>,
}

pub fn set_authority(
    ctx: Context<SetAuthority>,
) -> Result<()> {
    let global = &mut ctx.accounts.global;
    global.authority = ctx.accounts.new_authority.key();

    Ok(())
}
