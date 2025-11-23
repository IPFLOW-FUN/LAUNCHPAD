use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Global {
    pub initialized: bool,

    pub authority: Pubkey,

    pub fee_bps: u16,

    pub token_price_up_bps: u16,

    pub withdraw_fee_bps: u16,

    pub token_total_supply: u64,

    pub token_investing_supply: u64,

    pub fee_recipient: Pubkey,

    pub lp_recipient: Pubkey,

    pub migration_caller: Pubkey,

    pub token_creator_reserve: u64,

    pub token_platform_reserve: u64,

    pub token_pool_reserve: u64,
}

#[account]
#[derive(Default)]
pub struct BondingCurve {
    pub sol_reserves: u64,

    pub token_reserves: u64,

    pub token_total_supply: u64,

    pub token_investing_supply: u64,

    pub token_investing_price: u64,

    pub token_investing_deadline: u64,

    pub token_launching_price: u64,

    pub withdraw_fee_bps: u16,

    pub withdraw_recipient: Pubkey,

    pub completed: bool,

    pub investing_start_at: u64,

    pub whitelisted: bool,

    pub merkle_root: [u8; 32],

    pub whitelist_start_at: u64,

    pub token_creator_reserve: u64,

    pub token_platform_reserve: u64,

    pub token_pool_reserve: u64,

    pub migrated: bool,

    pub withdrawed: bool,
}

#[account]
#[derive(Default)]
pub struct UserPurchase {
    pub user: Pubkey,

    pub mint: Pubkey,

    pub token_amount: u64,
}
