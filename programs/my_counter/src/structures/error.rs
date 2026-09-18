use anchor_lang::prelude::error_code;

#[error_code]
pub enum BankErrorCodes {
    #[msg("Only the state authority can update this state")]
    Unauthorized,

    #[msg("Marker trades is stopped")]
    IsStop,

    #[msg("Counter has reached the maximum value")]
    CounterOverflow,

    #[msg("Protocol fee cannot exceed 100% (10,000 bps)")]
    InvalidProtocolFee,

    #[msg("Insufficient balance for this operation")]
    InsufficientBalance,

    #[msg("Wrong token amount")]
    InvalidAmount,

    #[msg("Overflow bank amount")]
    MathOverflow,

    #[msg("Wrong bank vault check date")]
    SPLBankCheckError
}