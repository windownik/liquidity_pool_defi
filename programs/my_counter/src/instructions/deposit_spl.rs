use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use anchor_lang::solana_program::program::invoke;
use spl_token::instruction::transfer as spl_transfer;

use crate::state::bank::Bank;
use crate::structures::error::BankErrorCodes;
use crate::structures::market::Market;
use crate::structures::user_deposit::UserDeposit;


#[derive(Accounts)]
pub struct DepositSpl<'info> {
    #[account(
        seeds = [b"lending_market"],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ BankErrorCodes::SPLBankCheckError,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"bank", user_token_account.mint.as_ref(), market.key().as_ref()],
        bump = bank.bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"bank_vault", bank.key().as_ref()],
        bump = bank.vault_bump,
        constraint = bank_vault.owner == bank.key() @ BankErrorCodes::SPLBankCheckError,
    )]
    pub bank_vault: Account<'info, TokenAccount>,


    #[account(
    mut,
    seeds = [b"user_deposit", market.key().as_ref(), bank.key().as_ref(), user.key().as_ref()],
    bump,
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    pub token_program: Program<'info, Token>,
}


pub fn process_deposit_spl_tokens(
    ctx: Context<DepositSpl>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, BankErrorCodes::InvalidAmount);
    let market = & ctx.accounts.market;
    require!(!market.is_paused, BankErrorCodes::IsStop);
    let bank = &mut ctx.accounts.bank;
    let bank_vault = &mut ctx.accounts.bank_vault;
    let user_deposit = &mut ctx.accounts.user_deposit;

    // Create transaction
    let ix = spl_transfer(
        ctx.accounts.token_program.key,
        ctx.accounts.user_token_account.to_account_info().key,
        ctx.accounts.bank_vault.to_account_info().key,
        ctx.accounts.user.to_account_info().key,
        // for multi signers
        &[],
        amount,
    )?;

    // Transfer SPL
    invoke(
        &ix,
        &[
            ctx.accounts.user_token_account.to_account_info(),
            ctx.accounts.bank_vault.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;

    bank.total_deposits = bank.total_deposits.checked_add(amount).ok_or(BankErrorCodes::MathOverflow)?;
    user_deposit.amount = user_deposit.amount.checked_add(amount).ok_or(BankErrorCodes::MathOverflow)?;
    msg!("Deposit successful: {} tokens", amount);
    Ok(())
}