use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::bank::Bank;
use crate::structures::error::BankErrorCodes;
use crate::structures::market::Market;

#[derive(Accounts)]
pub struct CreateBank<'info> {
    #[account(
        seeds = [b"lending_market"],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = 8 + Bank::INIT_SPACE,
        seeds = [b"bank", market.key().as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        init,
        payer = admin,
        seeds = [b"vault", bank.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = bank,
        token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn process_create_bank(ctx: Context<CreateBank>) -> Result<()> {
    let bank = &mut ctx.accounts.bank;
    let market = & ctx.accounts.market;
    let admin = & ctx.accounts.admin;
    if market.admin != admin.key() {
        return Err(Error::from(BankErrorCodes::Unauthorized));
    }
    bank.market = ctx.accounts.market.key();
    bank.mint = ctx.accounts.mint.key();
    bank.vault = ctx.accounts.vault.key();
    bank.total_deposits = 0;
    bank.total_borrows = 0;
    bank.deposit_limit = 0;
    bank.bump = ctx.bumps.bank;
    bank.vault_bump = ctx.bumps.vault;

    msg!(
        "Bank created for mint {} with vault {}",
        bank.mint,
        bank.vault
    );
    Ok(())
}