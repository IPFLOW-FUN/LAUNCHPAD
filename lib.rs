#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

pub mod instructions;
use instructions::*;

mod constants;
mod errors;
mod state;
mod events;
mod utils;

declare_id!("3v8WEa92iJjbbTJRTgGzZbwDQCWMassUZmoE4kgbLUev");

#[program]
pub mod memechef {
    use super::*;

    /// Creates the global state.
    pub fn initialize(
        ctx: Context<Initialize>,
    ) -> Result<()> {
        instructions::initialize(ctx)
    }

    /// Sets the global state parameters.
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
        instructions::set_params(ctx, fee_bps, token_price_up_bps, withdraw_fee_bps, token_total_supply, token_investing_supply, token_creator_reserve, token_platform_reserve, token_pool_reserve)
    }

    /// Sets the new authority of global state.
    pub fn set_authority(
        ctx: Context<SetAuthority>,
    ) -> Result<()> {
        instructions::set_authority(ctx)
    }

    /// Creates a new coin and bonding curve.
    pub fn create_token(
        ctx: Context<CreateToken>,
        token_name: String,
        token_symbol: String,
        token_uri: String,
        token_investing_price: u64,
        token_investing_deadline: u64,
        investing_start_at: u64,
        whitelisted: bool,
        merkle_root: [u8; 32],
        whitelist_start_at: u64,
    ) -> Result<()> {
        instructions::create_token(ctx, token_name, token_symbol, token_uri, token_investing_price, token_investing_deadline, investing_start_at, whitelisted, merkle_root, whitelist_start_at)
    }

    /// Buys tokens from a bonding curve.
    pub fn buy(
        ctx: Context<Buy>,
        amount: u64,
        max_sol_cost: u64,
        merkle_proof: Option<Vec<[u8; 32]>>,
    ) -> Result<()> {
        instructions::buy(ctx, amount, max_sol_cost, merkle_proof)
    }

    /// Sells tokens into a bonding curve.
    pub fn sell(
        ctx: Context<Sell>,
        amount: u64,
        min_sol_output: u64,
    ) -> Result<()> {
        instructions::sell(ctx, amount, min_sol_output)
    }

    /// Withdraws funds after bonding curve completes (must be called before migrate_liquidity).
    pub fn withdraw(
        ctx: Context<Withdraw>,
    ) -> Result<()> {
        instructions::withdraw(ctx)
    }

    /// Allows the admin to migrate liquidity once the bonding curve completes.
    pub fn migrate_liquidity(
        ctx: Context<MigrateLiquidity>,
    ) -> Result<()> {
        instructions::migrate_liquidity(ctx)
    }

    pub fn migrate_liquidity_fallback(
        ctx: Context<MigrateLiquidityFallback>,
    ) -> Result<()> {
        instructions::migrate_liquidity_fallback(ctx)
    }

    pub fn proxy_swap_base_input(
        ctx: Context<ProxySwapBaseInput>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        instructions::proxy_swap_base_input(ctx, amount_in, minimum_amount_out)
    }

    pub fn proxy_swap_base_output(
        ctx: Context<ProxySwapBaseOutput>,
        max_amount_in: u64,
        amount_out: u64,
    ) -> Result<()> {
        instructions::proxy_swap_base_output(ctx, max_amount_in, amount_out)
    }

    /// Sets the migrated status of a bonding curve (admin only).
    pub fn set_migrated(
        ctx: Context<SetMigrated>,
    ) -> Result<()> {
        instructions::set_migrated(ctx)
    }

    /// Updates the merkle root of a bonding curve (admin only).
    pub fn set_merkle_root(
        ctx: Context<SetMerkleRoot>,
        new_merkle_root: [u8; 32],
    ) -> Result<()> {
        instructions::set_merkle_root(ctx, new_merkle_root)
    }

    /// Allows users to claim their purchased tokens after migration.
    pub fn claim(
        ctx: Context<Claim>,
    ) -> Result<()> {
        instructions::claim(ctx)
    }
}
