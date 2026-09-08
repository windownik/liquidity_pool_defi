

use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserBank {
    pub user: Pubkey,
    pub balance: u64,
    pub bump: u8,
}