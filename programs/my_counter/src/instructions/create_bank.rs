use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::bank::Bank;
use crate::structures::error::BankErrorCodes;
use crate::structures::market::Market;

#[derive(Accounts)]
pub struct CreateBank<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"lending_market"],
        constraint = market.admin == admin.key() @ BankErrorCodes::Unauthorized,
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = 8 + Bank::INIT_SPACE,
        seeds = [b"bank",  mint.key().as_ref(), market.key().as_ref(),],
        bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        init,
        payer = admin,
        seeds = [b"bank_vault", bank.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = bank,
    )]
    pub bank_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn process_create_bank(ctx: Context<CreateBank>) -> Result<()> {
    let bank = &mut ctx.accounts.bank;
    bank.market = ctx.accounts.market.key();
    bank.mint = ctx.accounts.mint.key();
    bank.vault = ctx.accounts.bank_vault.key();
    bank.total_deposits = 0;
    bank.total_borrows = 0;
    bank.deposit_limit = 0;
    bank.bump = ctx.bumps.bank;
    bank.vault_bump = ctx.bumps.bank_vault;

    msg!(
        "Bank created for mint {} with vault {}",
        bank.mint,
        bank.vault
    );
    Ok(())
}