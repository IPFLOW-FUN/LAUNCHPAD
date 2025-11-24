use {
    crate::{constants::*, errors::Errors, state::*},
    anchor_lang::prelude::*, std::mem::size_of,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = size_of::<Global>() + 8,
        seeds = [GLOBAL_SEED.as_ref()],
        bump,
        constraint = global.initialized == false @ Errors::AlreadyInitialized,
    )]
    pub global: Box<Account<'info, Global>>,

    pub system_program: Program<'info, System>,
}

pub fn initialize(
    ctx: Context<Initialize>,
) -> Result<()> {
    let global = &mut ctx.accounts.global;
    global.initialized = true;
    global.authority = ctx.accounts.payer.key();

    Ok(())
}
