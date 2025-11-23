use {
    crate::{constants::*, errors::Errors, events::MigrateFallbackEvent, state::{BondingCurve, Global}},
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, SyncNative, Token, TokenAccount},
    },
};

#[derive(Accounts)]
pub struct MigrateLiquidityFallback<'info> {
    #[account(
        seeds = [GLOBAL_SEED.as_ref()],
        bump,
    )]
    pub global: Box<Account<'info, Global>>,

    #[account()]
    pub mint: Account<'info, Mint>,

    #[account()]
    pub native_mint: Account<'info, Mint>,

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

    /// CHECK: Address validated using constraint
    #[account(
        mut,
        address = global.lp_recipient @ Errors::InvalidLpRecipient
    )]
    pub lp_recipient: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = caller,
        associated_token::mint = mint,
        associated_token::authority = lp_recipient,
    )]
    pub lp_recipient_token: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = caller,
        associated_token::mint = native_mint,
        associated_token::authority = lp_recipient,
    )]
    pub lp_recipient_native_token: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub caller: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn migrate_liquidity_fallback(
    ctx: Context<MigrateLiquidityFallback>,
) -> Result<()> {
    require!(ctx.accounts.caller.key() == ctx.accounts.global.migration_caller, Errors::NotAuthorized);
    require!(ctx.accounts.bonding_curve.completed == true, Errors::BondingCurveNotComplete);
    require!(ctx.accounts.bonding_curve.withdrawed == true, Errors::BondingCurveNotWithdrawed);
    require!(ctx.accounts.bonding_curve.migrated == false, Errors::BondingCurveAlreadyMigrated);

    let clock: Clock = Clock::get()?;

    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let token_amount = bonding_curve.token_reserves;
    let sol_amount = bonding_curve.sol_reserves;

    require!(ctx.accounts.bonding_curve_vault.lamports() >= sol_amount, Errors::BondingCurveAlreadyMigrated);

    if sol_amount > 0 {
        let vault_seeds = &[
            BONDING_CURVE_VAULT_SEED.as_bytes(),
            &ctx.accounts.mint.key().to_bytes(),
            &[ctx.bumps.bonding_curve_vault],
        ];
        let vault_signer_seeds = &[&vault_seeds[..]];

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.bonding_curve_vault.to_account_info(),
                    to: ctx.accounts.lp_recipient_native_token.to_account_info(),
                },
                vault_signer_seeds,
            ),
            sol_amount,
        )?;

        token::sync_native(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                SyncNative {
                    account: ctx.accounts.lp_recipient_native_token.to_account_info(),
                },
            ),
        )?;
    }

    if token_amount > 0 {
        let bonding_curve_seeds = &[
            BONDING_CURVE_SEED.as_bytes(),
            &ctx.accounts.mint.key().to_bytes(),
            &[ctx.bumps.bonding_curve],
        ];
        let signer_seeds = &[&bonding_curve_seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.associated_bonding_curve.to_account_info(),
                    to: ctx.accounts.lp_recipient_token.to_account_info(),
                    authority: bonding_curve.to_account_info(),
                },
                signer_seeds,
            ),
            token_amount,
        )?;
    }

    bonding_curve.token_reserves = 0;
    bonding_curve.sol_reserves = 0;
    bonding_curve.migrated = true;

    emit!(MigrateFallbackEvent {
        user: ctx.accounts.caller.key(),
        mint: ctx.accounts.mint.key(),
        bonding_curve: ctx.accounts.bonding_curve.key(),
        token_amount,
        sol_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
