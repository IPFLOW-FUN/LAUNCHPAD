use {
    crate::{constants::*, errors::Errors, events::*, state::*},
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, Token},
    },
};
use solana_program::clock::Clock;

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(
        seeds = [GLOBAL_SEED.as_ref()],
        bump,
    )]
    pub global: Box<Account<'info, Global>>,

    #[account()]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            BONDING_CURVE_SEED.as_ref(),
            mint.key().as_ref(),
        ],
        bump,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        mut,
        seeds = [
            BONDING_CURVE_VAULT_SEED.as_ref(),
            mint.key().as_ref(),
        ],
        bump,
    )]
    pub bonding_curve_vault: SystemAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            USER_PURCHASE_SEED.as_ref(),
            mint.key().as_ref(),
            payer.key().as_ref(),
        ],
        bump,
    )]
    pub user_purchase: Box<Account<'info, UserPurchase>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn sell(
    ctx: Context<Sell>,
    amount: u64,
    min_sol_output: u64,
) -> Result<()> {
    require!(amount > 0 && min_sol_output > 0, Errors::InvalidValue);

    // Getting clock
    let clock: Clock = Clock::get()?;
    let now = clock.unix_timestamp.try_into().unwrap();

    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let user_purchase = &mut ctx.accounts.user_purchase;

    // Enhanced validations
    require!(bonding_curve.completed == false, Errors::BondingCurveComplete);

    // Selling is only allowed after the investing deadline has passed
    require!(bonding_curve.token_investing_deadline <= now, Errors::BondingCurveNotEnded);

    // Check if user has enough purchase amount to sell
    require!(user_purchase.token_amount >= amount, Errors::InsufficientBalance);

    let token_decimals = *&ctx.accounts.mint.decimals;

    let token_amount = amount;
    // Safe math for SOL amount calculation
    let sol_amount = (token_amount as u128)
        .checked_mul(bonding_curve.token_investing_price as u128)
        .and_then(|x| x.checked_div(10u128.pow(token_decimals.into())))
        .and_then(|x| u64::try_from(x).ok())
        .ok_or(Errors::MathOverflow)?;

    require!(sol_amount >= min_sol_output, Errors::TooLittleSolReceived);
    require!(bonding_curve.sol_reserves >= sol_amount, Errors::InvalidValue);

    // Update reserves with safe math
    bonding_curve.sol_reserves = bonding_curve.sol_reserves
        .checked_sub(sol_amount)
        .ok_or(Errors::MathOverflow)?;
    bonding_curve.token_reserves = bonding_curve.token_reserves
        .checked_add(token_amount)
        .ok_or(Errors::MathOverflow)?;

    // Reduce user purchase amount instead of token transfer
    user_purchase.token_amount = user_purchase.token_amount
        .checked_sub(token_amount)
        .ok_or(Errors::MathOverflow)?;

    let vault_seeds = &[
        BONDING_CURVE_VAULT_SEED.as_bytes(),
        &ctx.accounts.mint.key().to_bytes(),
        &[ctx.bumps.bonding_curve_vault],
    ];
    let vault_signer_seeds = &[&vault_seeds[..]];

    // Return SOL to user
    system_program::transfer(
        CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
                from: ctx.accounts.bonding_curve_vault.to_account_info().clone(),
                to: ctx.accounts.payer.to_account_info().clone(),
            },
            vault_signer_seeds,
        ),
        sol_amount,
    )?;

    emit!(TradeEvent {
        mint: ctx.accounts.mint.key(),
        sol_amount: sol_amount,
        token_amount: token_amount,
        fee_amount: 0,
        is_buy: false,
        user: ctx.accounts.payer.key(),
        timestamp: clock.unix_timestamp,
    });
    msg!("Sell successfully.");

  Ok(())
}
