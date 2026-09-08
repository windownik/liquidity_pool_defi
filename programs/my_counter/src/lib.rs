pub mod constants;

pub mod state;
pub mod bank_with_pda;
pub mod structures;

use anchor_lang::prelude::*;

pub use bank_with_pda::*;
pub use structures::currency::*;

declare_id!("3gW4EMz31zs6HMxyoCWUb5mxXeLj4V3Ur2H41VvPWPMM");

#[program]
pub mod my_liquidity_bank {
    use crate::bank_with_pda::initialize_market::{process_initialize_market, InitializeMarket};
    use crate::update_is_paused_status::{update_is_paused, UpdateIsPaused};
    use crate::structures::currency::Currency;
    use super::*;

    pub fn initialize_market(ctx: Context<InitializeMarket>, protocol_fee_bps: u16,) -> Result<()> {
        process_initialize_market(ctx, protocol_fee_bps)
    }

    pub fn deposit(ctx: Context<InitializeMarket>, amount: u64, currency: Currency) -> Result<()> {
        todo!();
    }

    // pub fn withdraw(ctx: Context<InitializeMarket>, amount: u64, currency: Currency) -> Result<()> {
    //     todo!();
    // }
    //
    // pub fn borrow(ctx: Context<InitializeMarket>, amount: u64, currency: Currency) -> Result<()> {
    //     todo!();
    // }

    // pub fn repay(ctx: Context<Repay>, amount: u64, currency: Currency) -> Result<()> {
    //     todo!();
    // }
    //
    // pub fn liquidate(ctx: Context<Liquidate>, amount: u64, currency: Currency) -> Result<()> {
    //     todo!();
    // }
    //
    // pub fn update_admin(ctx: Context<UpdateAdmin>, new_admin: Pubkey) -> Result<()> {
    //     todo!();
    // }
    //
    // pub fn withdraw_protocol_fees(ctx: Context<WithdrawProtocolFees>, amount: u64) -> Result<()> {
    //     todo!();
    // }

    pub fn change_is_paused(ctx: Context<UpdateIsPaused>, is_paused: bool,) -> Result<()> {
        update_is_paused(ctx, is_paused)
    }
}



