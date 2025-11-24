use {
    crate::{constants::*, errors::Errors, events::ClaimEvent, state::*},
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, Token, TokenAccount},
    },
};

#[derive(Accounts)]
pub struct Claim<'info> {
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
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            USER_PURCHASE_SEED.as_ref(),
            mint.key().as_ref(),
            user.key().as_ref(),
        ],
        bump,
    )]
    pub user_purchase: Box<Account<'info, UserPurchase>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn claim(ctx: Context<Claim>) -> Result<()> {
    let bonding_curve = &ctx.accounts.bonding_curve;
    let user_purchase = &mut ctx.accounts.user_purchase;

    // Check if migrated
    require!(bonding_curve.migrated == true, Errors::NotMigrated);

    // Check if user has purchase record
    require!(user_purchase.token_amount > 0, Errors::NoPurchaseRecord);

    let token_amount = user_purchase.token_amount;

    // Transfer token from bonding_curve to user
    let seeds = &[
        BONDING_CURVE_SEED.as_bytes(),
        &ctx.accounts.mint.key().to_bytes(),
        &[ctx.bumps.bonding_curve],
    ];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.associated_bonding_curve.to_account_info().clone(),
                to: ctx.accounts.user_token_account.to_account_info().clone(),
                authority: bonding_curve.to_account_info(),
            },
            signer_seeds,
        ),
        token_amount,
    )?;

    // Reset token_amount to 0 after claiming
    user_purchase.token_amount = 0;

    msg!("User {} claimed {} tokens", ctx.accounts.user.key(), token_amount);

    emit!(ClaimEvent {
        user: ctx.accounts.user.key(),
        mint: ctx.accounts.mint.key(),
        bonding_curve: bonding_curve.key(),
        token_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
