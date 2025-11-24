use anchor_lang::prelude::*;

/// Event of token creation
#[event]
pub struct CreateEvent {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub user: Pubkey,
    pub timestamp: i64,
}

/// Event of trade order
#[event]
pub struct TradeEvent {
    pub mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub fee_amount: u64,
    pub is_buy: bool,
    pub user: Pubkey,
    pub timestamp: i64,
}

/// Event of bonding curve completed
#[event]
pub struct CompleteEvent {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub timestamp: i64,
}

/// Event of token withdraw
#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub token_amount: u64,
    pub sol_amount: u64,
    pub timestamp: i64,
}

/// Event of global state settings
#[event]
pub struct SetParamsEvent {
    pub fee_recipient: Pubkey,
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
    pub fee_basis_points: u64,
}

/// Event of token migration
#[event]
pub struct MigrateEvent {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub token_amount: u64,
    pub sol_amount: u64,
    pub timestamp: i64,
}

/// Event of fallback token migration
#[event]
pub struct MigrateFallbackEvent {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub token_amount: u64,
    pub sol_amount: u64,
    pub timestamp: i64,
}

/// Event of proxy trade on raydium bia memechef
#[event]
pub struct ProxyTradeEvent {
    pub market: Pubkey,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub input_token_amount: u64,
    pub output_token_amount: u64,
    pub fee_amount: u64,
    pub fee_recipient_token_account: Pubkey,
    pub timestamp: i64,
}

/// Event of token claim
#[event]
pub struct ClaimEvent {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub token_amount: u64,
    pub timestamp: i64,
}
