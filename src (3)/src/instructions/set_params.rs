use {
    crate::{constants::*, errors::Errors, state::*},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct SetParams<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_SEED.as_ref()],
        bump,
        constraint = global.initialized == true @ Errors::NotInitialized,
        constraint = global.authority == payer.key() @ Errors::NotAuthorized,
    )]
    pub global: Box<Account<'info, Global>>,

    /// CHECK: Address validated using constraint
    #[account(
        mut,
    )]
    pub fee_recipient: UncheckedAccount<'info>,

    /// CHECK: Address validated using constraint
    #[account(
        mut,
    )]
    pub lp_recipient: UncheckedAccount<'info>,

    /// CHECK: Address validated using constraint
    #[account(
        mut,
    )]
    pub migration_caller: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn set_params(
    ctx: Context<SetParams>,
    fee_bps: u16,
    token_price_up_bps: u16,
    withdraw_fee_bps: u16,
    token_total_supply: u64,
    token_investing_supply: u64,
    token_creator_reserve: u64,
    token_platform_reserve: u64,
    token_pool_reserve: u64,
) -> Result<()> {
    require!(fee_bps < 10000, Errors::InvalidValue);
    require!(token_price_up_bps >= 10000, Errors::InvalidValue);
    require!(withdraw_fee_bps < 10000, Errors::InvalidValue);

    // Validate that the sum of all token allocations does not exceed total supply
    let total_allocation = token_investing_supply
        .checked_add(token_creator_reserve)
        .and_then(|sum| sum.checked_add(token_platform_reserve))
        .and_then(|sum| sum.checked_add(token_pool_reserve))
        .ok_or(Errors::MathOverflow)?;

    require!(total_allocation <= token_total_supply, Errors::InvalidValue);

    // Invariant I2: pool SOL requirement must not exceed raised SOL
    // token_pool_reserve * token_price_up_bps <= token_investing_supply * BASE_POINTS
    let lhs = (token_pool_reserve as u128)
        .checked_mul(token_price_up_bps as u128)
        .ok_or(Errors::MathOverflow)?;
    let rhs = (token_investing_supply as u128)
        .checked_mul(BASE_POINTS as u128)
        .ok_or(Errors::MathOverflow)?;
    require!(lhs <= rhs, Errors::InvalidValue);

    let global = &mut ctx.accounts.global;
    global.fee_bps = fee_bps;
    global.token_price_up_bps = token_price_up_bps;
    global.withdraw_fee_bps = withdraw_fee_bps;
    global.token_total_supply = token_total_supply;
    global.token_investing_supply = token_investing_supply;
    global.fee_recipient = ctx.accounts.fee_recipient.key();
    global.lp_recipient = ctx.accounts.lp_recipient.key();
    global.migration_caller = ctx.accounts.migration_caller.key();
    global.token_creator_reserve = token_creator_reserve;
    global.token_platform_reserve = token_platform_reserve;
    global.token_pool_reserve = token_pool_reserve;

    Ok(())
}
