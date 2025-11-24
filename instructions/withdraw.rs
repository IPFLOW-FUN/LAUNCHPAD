use {
    crate::{constants::*, errors::Errors, state::*},
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, Token, TokenAccount},
    },
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [GLOBAL_SEED.as_ref()],
        bump,
    )]
    pub global: Box<Account<'info, Global>>,

    /// CHECK: Address validated using constraint
    #[account(
        mut,
        address = global.fee_recipient @ Errors::InvalidFeeRecipient
    )]
    pub fee_recipient: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = fee_recipient,
    )]
    pub associated_fee_recipient: Box<Account<'info, TokenAccount>>,

    /// CHECK: Address validated using constraint
    #[account(
        mut,
        address = bonding_curve.withdraw_recipient @ Errors::InvalidWithdrawRecipient
    )]
    pub withdraw_recipient: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = withdraw_recipient,
    )]
    pub associated_withdraw_recipient: Box<Account<'info, TokenAccount>>,

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

    #[account(
        mut,
        seeds = [
            BONDING_CURVE_VAULT_SEED.as_ref(),
            mint.key().as_ref(),
        ],
        bump,
    )]
    pub bonding_curve_vault: SystemAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Box<Account<'info, TokenAccount>>,

    /// CHECK: Incinerator address
    #[account(address = crate::constants::INCINERATOR)]
    pub blackhole: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = blackhole,
    )]
    pub associated_blackhole: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub caller: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    require!(ctx.accounts.caller.key() == ctx.accounts.global.migration_caller, Errors::NotAuthorized);

    let bonding_curve = &mut ctx.accounts.bonding_curve;

    require!(bonding_curve.completed == true, Errors::BondingCurveNotComplete);
    require!(bonding_curve.withdrawed == false, Errors::BondingCurveAlreadyWithdrawed);

    let token_decimals = ctx.accounts.mint.decimals;

    // Prepare signer seeds for token operations
    let seeds = &[
        BONDING_CURVE_SEED.as_bytes(),
        &ctx.accounts.mint.key().to_bytes(),
        &[ctx.bumps.bonding_curve],
    ];
    let signer_seeds = &[&seeds[..]];

    // Calculate base token reduce for liquidity migration
    let final_token_reserves: u64 = bonding_curve.token_pool_reserve;
    // Total token reduce includes creator reserve, platform reserve, and pool reserve
    let token_burn = bonding_curve.token_reserves - final_token_reserves - bonding_curve.token_creator_reserve - bonding_curve.token_platform_reserve;

    let final_sol_reserves = (final_token_reserves as u128 * bonding_curve.token_launching_price as u128 / 10u128.pow(token_decimals.into())) as u64;
    let sol_withdraw = bonding_curve.sol_reserves - final_sol_reserves;
    let sol_fee = sol_withdraw * bonding_curve.withdraw_fee_bps as u64 / BASE_POINTS;
    let sol_creator = sol_withdraw - sol_fee;

    if token_burn > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.associated_bonding_curve.to_account_info(),
                    to: ctx.accounts.associated_blackhole.to_account_info(),
                    authority: bonding_curve.to_account_info(),
                },
                signer_seeds,
            ),
            token_burn,
        )?;
    }

    let vault_seeds = &[
        BONDING_CURVE_VAULT_SEED.as_bytes(),
        &ctx.accounts.mint.key().to_bytes(),
        &[ctx.bumps.bonding_curve_vault],
    ];
    let vault_signer_seeds = &[&vault_seeds[..]];

    // Withdraw to creator
    system_program::transfer(
        CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
                from: ctx.accounts.bonding_curve_vault.to_account_info().clone(),
                to: ctx.accounts.withdraw_recipient.to_account_info().clone(),
            },
            vault_signer_seeds,
        ),
        sol_creator,
    )?;
    // Withdraw fees to platform
    system_program::transfer(
        CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
                from: ctx.accounts.bonding_curve_vault.to_account_info().clone(),
                to: ctx.accounts.fee_recipient.to_account_info().clone(),
            },
            vault_signer_seeds,
        ),
        sol_fee,
    )?;

    // Transfer reserved tokens to issuer for marketing/airdrop
    if bonding_curve.token_creator_reserve > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.associated_bonding_curve.to_account_info().clone(),
                    to: ctx.accounts.associated_withdraw_recipient.to_account_info(),
                    authority: bonding_curve.to_account_info(),
                },
                signer_seeds,
            ),
            bonding_curve.token_creator_reserve,
        )?;
    }

    // Transfer platform reserved tokens to fee recipient
    if bonding_curve.token_platform_reserve > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.associated_bonding_curve.to_account_info().clone(),
                    to: ctx.accounts.associated_fee_recipient.to_account_info(),
                    authority: bonding_curve.to_account_info(),
                },
                signer_seeds,
            ),
            bonding_curve.token_platform_reserve,
        )?;
    }

    bonding_curve.sol_reserves = final_sol_reserves;
    bonding_curve.token_reserves = final_token_reserves;
    bonding_curve.withdrawed = true;

    msg!("Withdraw completed. Creator received: {} lamports, Platform fee: {} lamports", sol_creator, sol_fee);

    Ok(())
}
