use anchor_lang::prelude::*;

#[constant]
pub const GLOBAL_SEED: &str = "global";

#[constant]
pub const MINT_AUTHORITY_SEED: &str = "mint_authority";

#[constant]
pub const BONDING_CURVE_SEED: &str = "bonding_curve";

#[constant]
pub const BONDING_CURVE_VAULT_SEED: &str = "bonding_curve_vault";

#[constant]
pub const USER_PURCHASE_SEED: &str = "user_purchase";

pub const INCINERATOR: Pubkey = anchor_lang::solana_program::pubkey!("1nc1nerator11111111111111111111111111111111");

#[constant]
pub const BASE_POINTS: u64 = 10000;
