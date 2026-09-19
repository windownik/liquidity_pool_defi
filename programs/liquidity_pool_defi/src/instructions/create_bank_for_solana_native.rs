use anchor_lang::prelude::*;
use crate::state::bank::Bank;
use crate::structures::error::BankErrorCodes;
use crate::structures::market::Market;
use anchor_spl::token::spl_token::native_mint;


#[derive(Accounts)]
pub struct CreateBankNativeSolana<'info> {
    #[account(
        seeds = [b"lending_market"],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    #[account(
        init,
        payer = admin,
        space = 8 + Bank::INIT_SPACE,
        seeds = [b"bank_solana_native", market.key().as_ref()],
        bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn process_create_bank_for_native_solana(ctx: Context<CreateBankNativeSolana>) -> Result<()> {

    let market = & ctx.accounts.market;
    let admin = & ctx.accounts.admin;
    if market.admin != admin.key() {
        return Err(Error::from(BankErrorCodes::Unauthorized));
    }

    let bank = &mut ctx.accounts.bank;

    bank.market = ctx.accounts.market.key();
    bank.mint = native_mint::ID;
    bank.vault = bank.key();
    bank.total_deposits = 0;
    bank.total_borrows = 0;
    bank.deposit_limit = 0;
    bank.bump = ctx.bumps.bank;
    bank.vault_bump = ctx.bumps.bank;

    msg!(
        "Bank created for mint {} with vault {}",
        bank.mint,
        bank.vault
    );
    Ok(())
}