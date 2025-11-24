use {
    crate::{constants::*, errors::Errors, events::*, state::*, utils::*},
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, Token},
    },
    std::mem::size_of,
};
use solana_program::clock::Clock;

#[derive(Accounts)]
pub struct Buy<'info> {
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
        init_if_needed,
        payer = payer,
        space = size_of::<UserPurchase>() + 8,
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

pub fn buy(
    ctx: Context<Buy>,
    amount: u64,
    max_sol_cost: u64,
    merkle_proof: Option<Vec<[u8; 32]>>,
) -> Result<()> {
    require!(amount > 0 && max_sol_cost > 0, Errors::InvalidValue);

    let bonding_curve = &mut ctx.accounts.bonding_curve;

    // Getting clock
    let clock: Clock = Clock::get()?;
    let now = clock.unix_timestamp.try_into().unwrap();

    require!(bonding_curve.completed == false, Errors::BondingCurveComplete);
    // Buying is not allowed when the bonding_curve is not completed and the deadline is reached
    require!(bonding_curve.token_investing_deadline > now, Errors::BondingCurveEnded);

    // Time and whitelist validation
    if now < bonding_curve.investing_start_at {
        // Before public sale starts, only whitelisted users can buy
        require!(bonding_curve.whitelisted, Errors::BondingCurveNotStart);
        require!(now >= bonding_curve.whitelist_start_at, Errors::BondingCurveNotStart);

        // Merkle proof is required during whitelist period
        let proof = merkle_proof.as_ref().ok_or(Errors::MerkleProofMissing)?;

        // Generate leaf hash from user address only
        let leaf = get_leaf_hash(&ctx.accounts.payer.key(), None);

        require!(
            verify_merkle_proof(proof, &bonding_curve.merkle_root, leaf),
            Errors::NotWhitelisted
        );
    }
    
    
    let token_decimals = *&ctx.accounts.mint.decimals;

    let mut token_amount = amount;
    let mut completed = false;

    // Improved state consistency check for remaining investment amount
    let tokens_sold = bonding_curve.token_total_supply
        .checked_sub(bonding_curve.token_reserves)
        .ok_or(Errors::MathOverflow)?;
    let investing_amount_left = bonding_curve.token_investing_supply
        .checked_sub(tokens_sold)
        .ok_or(Errors::MathOverflow)?;
    
    if amount >= investing_amount_left {
        token_amount = investing_amount_left;
        completed = true;
    }
    // Safe math to prevent overflow
    let mut sol_amount = (token_amount as u128)
        .checked_mul(bonding_curve.token_investing_price as u128)
        .and_then(|x| x.checked_div(10u128.pow(token_decimals.into())))
        .and_then(|x| u64::try_from(x).ok())
        .ok_or(Errors::MathOverflow)?;

    // Handle edge case: when remaining tokens are very small, sol_amount might round to 0
    // In this case, charge minimum 1 lamport to allow completion
    if sol_amount == 0 {
        sol_amount = 1;
    }

    require!(sol_amount <= max_sol_cost, Errors::TooMuchSolRequired);

    bonding_curve.sol_reserves += sol_amount;
    bonding_curve.token_reserves -= token_amount;

    // Transfer SOL from payer to bonding_curve
    system_program::transfer(
        CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
                from: ctx.accounts.payer.to_account_info().clone(),
                to: ctx.accounts.bonding_curve_vault.to_account_info().clone(),
            },
        ),
        sol_amount
    )?;

    // Record user purchase instead of immediate token transfer
    let user_purchase = &mut ctx.accounts.user_purchase;
    if user_purchase.user == Pubkey::default() {
        user_purchase.user = ctx.accounts.payer.key();
        user_purchase.mint = ctx.accounts.mint.key();
        user_purchase.token_amount = 0;
    }
    user_purchase.token_amount = user_purchase
        .token_amount
        .checked_add(token_amount)
        .ok_or(Errors::MathOverflow)?;

    emit!(TradeEvent {
        mint: ctx.accounts.mint.key(),
        sol_amount: sol_amount,
        token_amount: token_amount,
        fee_amount: 0,
        is_buy: true,
        user: ctx.accounts.payer.key(),
        timestamp: clock.unix_timestamp,
    });

    // The bonding curve completed
    if completed {
        bonding_curve.completed = true;

        emit!(CompleteEvent {
            user: ctx.accounts.payer.key(),
            mint: ctx.accounts.mint.key(),
            bonding_curve: bonding_curve.key(),
            timestamp: clock.unix_timestamp,
        });
        msg!("The bonding curve has completed.");
    }

    Ok(())
}
