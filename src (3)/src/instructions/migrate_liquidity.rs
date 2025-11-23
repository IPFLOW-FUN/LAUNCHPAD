use {
    crate::{constants::*, errors::Errors, events::MigrateEvent, state::{BondingCurve, Global}},
    anchor_lang::{prelude::*, system_program},
    anchor_spl::{
        associated_token::{AssociatedToken, Create},
        token::{self, Mint, Token, TokenAccount, Transfer},
        token_interface::{Mint as MintInterface, TokenAccount as TokenAccountInterface, TokenInterface},
    },
    raydium_cp_swap::{
        cpi,
        program::RaydiumCpSwap,
        states::{AmmConfig, OBSERVATION_SEED, POOL_LP_MINT_SEED, POOL_SEED, POOL_VAULT_SEED},
    },
};

#[derive(Accounts)]
pub struct MigrateLiquidity<'info> {
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

    /// CHECK: Will be created during instruction execution after LP mint is initialized
    #[account(mut)]
    pub lp_recipient_token_lp: UncheckedAccount<'info>,

    pub cp_swap_program: Program<'info, RaydiumCpSwap>,
    /// Address paying to create the pool. Can be anyone
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Which config the pool belongs to.
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            raydium_cp_swap::AUTH_SEED.as_bytes(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Initialize an account to store the pool state, init by cp-swap
    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_0_mint.key().as_ref(),
            token_1_mint.key().as_ref(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub pool_state: UncheckedAccount<'info>,

    /// Token_0 mint, the key must smaller then token_1 mint.
    #[account(
        constraint = token_0_mint.key() < token_1_mint.key(),
        mint::token_program = token_0_program,
    )]
    pub token_0_mint: Box<InterfaceAccount<'info, MintInterface>>,

    /// Token_1 mint, the key must grater then token_0 mint.
    #[account(
        mint::token_program = token_1_program,
    )]
    pub token_1_mint: Box<InterfaceAccount<'info, MintInterface>>,

    /// CHECK: pool lp mint, init by cp-swap
    #[account(
        mut,
        seeds = [
            POOL_LP_MINT_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub lp_mint: UncheckedAccount<'info>,

    /// payer token0 account
    #[account(
        mut,
        associated_token::mint = token_0_mint,
        associated_token::authority = creator,
    )]
    pub creator_token_0: Box<Account<'info, TokenAccount>>,

    /// creator token1 account
    #[account(
        mut,
        associated_token::mint = token_1_mint,
        associated_token::authority = creator,
    )]
    pub creator_token_1: Box<Account<'info, TokenAccount>>,

    /// CHECK: creator lp ATA token account, init by cp-swap
    #[account(mut)]
    pub creator_lp_token: UncheckedAccount<'info>,

    /// CHECK: Token_0 vault for the pool, init by cp-swap
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_0_mint.key().as_ref()
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub token_0_vault: UncheckedAccount<'info>,

    /// CHECK: Token_1 vault for the pool, init by cp-swap
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_1_mint.key().as_ref()
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub token_1_vault: UncheckedAccount<'info>,

    /// create pool fee account
    #[account(
        mut,
        address= raydium_cp_swap::create_pool_fee_reveiver::id(),
    )]
    pub create_pool_fee: Box<InterfaceAccount<'info, TokenAccountInterface>>,

    /// CHECK: an account to store oracle observations, init by cp-swap
    #[account(
        mut,
        seeds = [
            OBSERVATION_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub observation_state: UncheckedAccount<'info>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Spl token program or token program 2022
    pub token_0_program: Interface<'info, TokenInterface>,
    /// Spl token program or token program 2022
    pub token_1_program: Interface<'info, TokenInterface>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

pub fn migrate_liquidity(
    ctx: Context<MigrateLiquidity>,
) -> Result<()> {
    require!(ctx.accounts.creator.key() == ctx.accounts.global.migration_caller, Errors::NotAuthorized);
    require!(ctx.accounts.bonding_curve.completed == true, Errors::BondingCurveNotComplete);
    require!(ctx.accounts.bonding_curve.withdrawed == true, Errors::BondingCurveNotWithdrawed);
    require!(ctx.accounts.bonding_curve.migrated == false, Errors::BondingCurveAlreadyMigrated);

    // Getting clock
    let clock: Clock = Clock::get()?;

    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let token_amount = bonding_curve.token_reserves;
    let sol_amount = bonding_curve.sol_reserves;

    require!(ctx.accounts.bonding_curve_vault.lamports() >= bonding_curve.sol_reserves, Errors::BondingCurveAlreadyMigrated);

    let open_time = 0;
    let init_amount_0 ;
    let init_amount_1;
    let creator_token_account;
    let creator_native_account;
    if ctx.accounts.token_0_mint.key() == ctx.accounts.native_mint.key() {
        init_amount_0 = sol_amount;
        init_amount_1 = token_amount;
        creator_native_account = &ctx.accounts.creator_token_0;
        creator_token_account = &ctx.accounts.creator_token_1;
    } else {
        init_amount_0 = token_amount;
        init_amount_1 = sol_amount;
        creator_native_account = &ctx.accounts.creator_token_1;
        creator_token_account = &ctx.accounts.creator_token_0;
    }

    // transfer sol from bonding_curve_vault to creator
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
                from: ctx.accounts.bonding_curve_vault.to_account_info().clone(),
                to: creator_native_account.to_account_info().clone(),
            },
            vault_signer_seeds,
        ),
        sol_amount,
    )?;
    // wrap sol to wsol
    token::sync_native(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::SyncNative {
            account: creator_native_account.to_account_info(),
        },
    ))?;

    // Transfer token from bonding_curve to creator
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
                to: creator_token_account.to_account_info().clone(),
                authority: bonding_curve.to_account_info(),
            },
            signer_seeds,
        ),
        token_amount,
    )?;

    // Create Pool
    let cpi_accounts = cpi::accounts::Initialize {
        creator: ctx.accounts.creator.to_account_info(),
        amm_config: ctx.accounts.amm_config.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        pool_state: ctx.accounts.pool_state.to_account_info(),
        token_0_mint: ctx.accounts.token_0_mint.to_account_info(),
        token_1_mint: ctx.accounts.token_1_mint.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        creator_token_0: ctx.accounts.creator_token_0.to_account_info(),
        creator_token_1: ctx.accounts.creator_token_1.to_account_info(),
        creator_lp_token: ctx.accounts.creator_lp_token.to_account_info(),
        token_0_vault: ctx.accounts.token_0_vault.to_account_info(),
        token_1_vault: ctx.accounts.token_1_vault.to_account_info(),
        create_pool_fee: ctx.accounts.create_pool_fee.to_account_info(),
        observation_state: ctx.accounts.observation_state.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        token_0_program: ctx.accounts.token_0_program.to_account_info(),
        token_1_program: ctx.accounts.token_1_program.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_context = CpiContext::new(ctx.accounts.cp_swap_program.to_account_info(), cpi_accounts);
    cpi::initialize(cpi_context, init_amount_0, init_amount_1, open_time)?;

    // Transfer the lp token to lp_recipient
    // Create LP recipient token account
    let lp_recipient_ata = anchor_spl::associated_token::get_associated_token_address(
        &ctx.accounts.lp_recipient.key(),
        &ctx.accounts.lp_mint.key()
    );
    require!(lp_recipient_ata == ctx.accounts.lp_recipient_token_lp.key(), Errors::InvalidLpRecipient);
    
    // Create the associated token account if not exists
    if ctx.accounts.lp_recipient_token_lp.data_is_empty() {
        anchor_spl::associated_token::create(
            CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                Create {
                    payer: ctx.accounts.creator.to_account_info(),
                    associated_token: ctx.accounts.lp_recipient_token_lp.to_account_info(),
                    authority: ctx.accounts.lp_recipient.to_account_info(),
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
            ),
        )?;
    }

    // Transfer liquidity tokens
    let user_token_lp_account = TokenAccount::try_deserialize(&mut ctx.accounts.creator_lp_token.clone().try_borrow_data()?.as_ref())?;
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.creator_lp_token.to_account_info(),
                to: ctx.accounts.lp_recipient_token_lp.to_account_info(),
                authority: ctx.accounts.creator.to_account_info(),
            },
        ),
        user_token_lp_account.amount,
    )?;

    bonding_curve.token_reserves = 0;
    bonding_curve.sol_reserves = 0;
    bonding_curve.migrated = true;

    msg!("Migrate completed.");

    emit!(MigrateEvent {
        user: ctx.accounts.creator.key(),
        mint: ctx.accounts.mint.key(),
        bonding_curve: ctx.accounts.bonding_curve.key(),
        token_amount: token_amount,
        sol_amount: sol_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
