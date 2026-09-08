use anchor_lang::prelude::*;


#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub enum Currency {
    Sol,
    USDC,
}