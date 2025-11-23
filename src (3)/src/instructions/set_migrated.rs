use {
    crate::{constants::*, errors::Errors, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::Mint,
};

#[derive(Accounts)]
pub struct SetMigrated<'info> {
    #[account(
        seeds = [GLOBAL_SEED.as_ref()],
        bump,
    )]
    pub global: Box<Account<'info, Global>>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            BONDING_CURVE_SEED.as_ref(),
            mint.key().as_ref(),
        ],
        bump,
    )]
    pub bonding_curve: Box<Account<'info, BondingCurve>>,

    #[account(mut)]
    pub caller: Signer<'info>,
}

pub fn set_migrated(
    ctx: Context<SetMigrated>,
) -> Result<()> {
    require!(ctx.accounts.caller.key() == ctx.accounts.global.migration_caller, Errors::NotAuthorized);
    require!(ctx.accounts.bonding_curve.withdrawed == true, Errors::BondingCurveNotWithdrawed);
    require!(ctx.accounts.bonding_curve.migrated == false, Errors::BondingCurveAlreadyMigrated);

    let bonding_curve = &mut ctx.accounts.bonding_curve;

    require!(bonding_curve.completed == true, Errors::BondingCurveNotComplete);

    bonding_curve.migrated = true;

    msg!("Bonding curve migrated status set to: true");

    Ok(())
}
