use anchor_lang::prelude::error_code;

#[error_code]
pub enum BankErrorCodes {
    #[msg("Caller is not authorized to perform this action")]
    Unauthorized,

    #[msg("Market is paused — all operations are temporarily disabled")]
    MarketPaused,

    #[msg("Protocol fee cannot exceed 100% (10,000 bps)")]
    InvalidProtocolFee,

    #[msg("Insufficient user balance for this operation")]
    InsufficientUserBalance,

    #[msg("Insufficient liquidity in the bank vault for this operation")]
    InsufficientVaultBalance,

    #[msg("Amount must be greater than zero")]
    InvalidAmount,

    #[msg("Arithmetic overflow — the resulting value does not fit in u64")]
    MathOverflow,

    #[msg("Token account validation failed: wrong owner, mint or vault authority")]
    InvalidTokenAccount,

    #[msg("Deposit amount exceeds the bank's configured deposit limit")]
    DepositLimitExceeded,

    #[msg("Cannot withdraw/borrow more than the user has available")]
    AmountExceedsAvailable,

    #[msg("User deposit account was not initialized for this bank")]
    UserDepositNotInitialized,

    #[msg("Bank for this mint does not exist on the current market")]
    BankNotFound,

    #[msg("Health factor is below the liquidation threshold")]
    UnhealthyPosition,
}