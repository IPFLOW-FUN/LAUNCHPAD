use {
    crate::{constants::*, errors::Errors, events::*, state::{BondingCurve, Global}},
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{
        associated_token::AssociatedToken,
        metadata::{create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3, Metadata},
        token::{mint_to, set_authority, Mint, MintTo, SetAuthority, Token, TokenAccount},
    },
    mpl_token_metadata::accounts::Metadata as mpl_metadata,
    spl_token::instruction::AuthorityType, std::mem::size_of,
};

#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(
        init,
        payer = payer,
        mint::decimals = 6,
        mint::authority = mint_authority,
    )]
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: Address validated using constraint
    #[account(
        seeds = [
            MINT_AUTHORITY_SEED.as_ref(),
        ],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = size_of::<BondingCurve>() + 8,
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
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = bonding_curve,
    )]
    pub associated_bonding_curve: Account<'info, TokenAccount>,

    /// CHECK: Address validated using constraint
    #[account(
        mut,
    )]
    pub withdraw_recipient: UncheckedAccount<'info>,

    #[account(
        seeds = [GLOBAL_SEED.as_ref()],
        bump,
        constraint = global.initialized == true @ Errors::NotInitialized,
    )]
    pub global: Box<Account<'info, Global>>,

    pub token_metadata_program: Program<'info, Metadata>,

    /// CHECK: Address validated using constraint
    #[account(
        mut,
        address=mpl_metadata::find_pda(&mint.key()).0
    )]
    pub metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

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
    require!(token_investing_price > 0 && token_investing_deadline > 0, Errors::InvalidValue);
    require!(whitelist_start_at <= investing_start_at, Errors::InvalidValue);
    require!(investing_start_at < token_investing_deadline, Errors::InvalidValue);
    require!(ctx.accounts.global.token_total_supply > 0, Errors::InvalidValue);
    require!(token_name.len() <= 32, Errors::NameTooLong);
    require!(token_symbol.len() <= 10, Errors::SymbolTooLong);
    require!(token_uri.len() <= 200, Errors::UriTooLong);

    // Getting clock
    let clock: Clock = Clock::get()?;
    let now = clock.unix_timestamp.try_into().unwrap();
    if whitelisted {
        require!(whitelist_start_at >= now, Errors::InvalidValue);
    }

    let global = &ctx.accounts.global;

    let seeds = &[
        MINT_AUTHORITY_SEED.as_bytes(),
        &[ctx.bumps.mint_authority],
    ];
    let signer_seeds = &[&seeds[..]];

    // Create token metadata
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.mint_authority.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.mint_authority.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        ),
        DataV2 {
            name: token_name.clone(),
            symbol: token_symbol.clone(),
            uri: token_uri.clone(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        false,
        true,
        None,
    )?;

    // Mint token
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.associated_bonding_curve.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        global.token_total_supply,
    )?;

    msg!("Token minted successfully.");

    // Transfer rent exemption amount to vault to ensure it can accept small purchases
    let rent_exempt_minimum = ctx.accounts.rent.minimum_balance(0);
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.bonding_curve_vault.to_account_info(),
            },
        ),
        rent_exempt_minimum,
    )?;

    msg!("Vault initialized with rent exemption.");

    // Revoke token mint authority to prevent further minting
    set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                current_authority: ctx.accounts.mint_authority.to_account_info(),
                account_or_mint: ctx.accounts.mint.to_account_info(),
            },
            signer_seeds,
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    let bonding_curve = &mut ctx.accounts.bonding_curve;
    bonding_curve.sol_reserves = 0;
    bonding_curve.token_reserves = global.token_total_supply;
    bonding_curve.token_total_supply = global.token_total_supply;
    bonding_curve.token_investing_supply = global.token_investing_supply;
    bonding_curve.token_investing_price = token_investing_price;
    bonding_curve.token_investing_deadline = token_investing_deadline;
    bonding_curve.token_launching_price = token_investing_price * global.token_price_up_bps as u64 / BASE_POINTS;
    bonding_curve.withdraw_fee_bps = global.withdraw_fee_bps;
    bonding_curve.withdraw_recipient = ctx.accounts.withdraw_recipient.key();
    bonding_curve.completed = false;
    bonding_curve.investing_start_at = investing_start_at;
    bonding_curve.whitelisted = whitelisted;
    bonding_curve.merkle_root = merkle_root;
    bonding_curve.whitelist_start_at = whitelist_start_at;
    bonding_curve.token_creator_reserve = global.token_creator_reserve;
    bonding_curve.token_platform_reserve = global.token_platform_reserve;
    bonding_curve.token_pool_reserve = global.token_pool_reserve;

    msg!("Bonding curve state saved successfully.");

    emit!(CreateEvent {
        name: token_name.clone(),
        symbol: token_symbol.clone(),
        uri: token_uri.clone(),
        mint: ctx.accounts.mint.key(),
        bonding_curve: ctx.accounts.bonding_curve.key(),
        user: ctx.accounts.payer.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
