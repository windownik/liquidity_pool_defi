use anchor_lang::prelude::*;

#[error_code]
pub enum BankErrorCodes {
    #[msg("Only the state authority can update this state")]
    Unauthorized,

    #[msg("Counter has reached the maximum value")]
    CounterOverflow,

    #[msg("Protocol fee cannot exceed 100% (10,000 bps)")]
    InvalidProtocolFee,

    #[msg("Insufficient balance for this operation")]
    InsufficientBalance,
}