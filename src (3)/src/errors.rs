use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("The given account is not authorized to execute this instruction.")]
    NotAuthorized,

    #[msg("The program is already initialized.")]
    AlreadyInitialized,

    #[msg("slippage: Too much SOL required to buy the given amount of tokens.")]
    TooMuchSolRequired,

    #[msg("slippage: Too little SOL received to sell the given amount of tokens.")]
    TooLittleSolReceived,

    #[msg("The mint does not match the bonding curve.")]
    MintDoesNotMatchBondingCurve,

    #[msg("The bonding curve has not start yet.")]
    BondingCurveNotStart,

    #[msg("The bonding curve has completed and liquidity migrated to raydium.")]
    BondingCurveComplete,

    #[msg("The bonding curve has not completed.")]
    BondingCurveNotComplete,
    
    #[msg("The bonding curve has ended.")]
    BondingCurveEnded,

    #[msg("The bonding curve has not ended.")]
    BondingCurveNotEnded,

    #[msg("The bonding curve has not launched.")]
    BondingCurveNotLaunched,

    #[msg("The bonding curve has been migrated.")]
    BondingCurveAlreadyMigrated,

    #[msg("The program is not initialized.")]
    NotInitialized,

    #[msg("Invalid fee value.")]
    InvalidFee,

    #[msg("Invalid value.")]
    InvalidValue,

    #[msg("Invalid fee recipient.")]
    InvalidFeeRecipient,

    #[msg("Invalid LP recipient.")]
    InvalidLpRecipient,

    #[msg("Invalid withdraw recipient.")]
    InvalidWithdrawRecipient,

    #[msg("The mint does not match.")]
    MintDoesNotMatch,

    #[msg("Invalid blackhole.")]
    InvalidBlackhole,

    #[msg("You are not whitelisted.")]
    NotWhitelisted,


    #[msg("Merkle proof is required for whitelist period.")]
    MerkleProofMissing,

    #[msg("Invalid merkle proof.")]
    InvalidMerkleProof,

    #[msg("Mathematical operation overflow.")]
    MathOverflow,

    #[msg("Insufficient token balance.")]
    InsufficientBalance,

    #[msg("Token not migrated yet.")]
    NotMigrated,

    #[msg("Already claimed.")]
    AlreadyClaimed,

    #[msg("No purchase record found.")]
    NoPurchaseRecord,

    #[msg("The bonding curve has been withdrawed.")]
    BondingCurveAlreadyWithdrawed,

    #[msg("The bonding curve has not been withdrawed.")]
    BondingCurveNotWithdrawed,

    #[msg("Token name is too long.")]
    NameTooLong,

    #[msg("Token symbol is too long.")]
    SymbolTooLong,

    #[msg("Token URI is too long.")]
    UriTooLong,
}
