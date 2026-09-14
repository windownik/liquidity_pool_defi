use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Bank {
    pub market: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub total_deposits: u64,
    pub total_borrows: u64,
    pub deposit_limit: u64,
    pub bump: u8,
    pub vault_bump: u8,
}