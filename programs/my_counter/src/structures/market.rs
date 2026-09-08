use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub admin: Pubkey,
    pub fee_receiver: Pubkey,
    pub reserves_count: u16,
    pub protocol_fee_bps: u16,
    pub is_paused: bool,
    pub version: u8,
    pub bump: u8,
    pub _reserved: [u8; 32],    
}