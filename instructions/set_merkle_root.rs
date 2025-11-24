use {
    crate::{constants::*, errors::Errors, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::Mint,
};

#[derive(Accounts)]
pub struct SetMerkleRoot<'info> {
    #[account(
        seeds = [GLOBAL_SEED.as_ref()],
        bump,
        constraint = global.initialized == true @ Errors::NotInitialized,
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
        constraint = payer.key() == global.authority @ Errors::NotAuthorized,
    )]
    pub payer: Signer<'info>,
}

pub fn set_merkle_root(
    ctx: Context<SetMerkleRoot>,
    new_merkle_root: [u8; 32],
) -> Result<()> {
    let bonding_curve = &mut ctx.accounts.bonding_curve;

    // Only allow updating merkle root before the sale completes
    require!(bonding_curve.completed == false, Errors::BondingCurveComplete);

    bonding_curve.merkle_root = new_merkle_root;

    msg!("Merkle root updated successfully");

    Ok(())
}
