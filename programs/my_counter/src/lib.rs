pub mod constants;

pub mod state;
pub mod instructions;
pub mod structures;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("13smfEHMmWM7Yaorw3kwXjeDq5LLppimtwVje6Jonz6j");

#[program]
pub mod my_liquidity_bank {
    use crate::instructions::initialize_market::{process_initialize_market, InitializeMarket};
    use crate::update_is_paused_status::{update_is_paused, UpdateIsPaused};
    use crate::deposit_sol::{process_deposit_sol, DepositSol};
    use crate::create_bank::{process_create_bank, CreateBank};
    use super::*;

    pub fn initialize_market(ctx: Context<InitializeMarket>, protocol_fee_bps: u16,) -> Result<()> {
        process_initialize_market(ctx, protocol_fee_bps)
    }

    pub fn deposit_native_sol(ctx: Context<DepositSol>, amount: u64,) -> Result<()> {
        process_deposit_sol(ctx, amount)
    }

    pub fn create_bank_for_token(ctx: Context<CreateBank>,) -> Result<()> {
        process_create_bank(ctx)
    }

    pub fn create_bank_for_native_sol(ctx: Context<CreateBank>,) -> Result<()> {
        process_create_bank(ctx)
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



