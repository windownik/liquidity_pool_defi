use anchor_lang::prelude::*;
use anchor_spl::token::{Mint};

use crate::state::bank::Bank;
use crate::structures::market::Market;
use crate::structures::user_deposit::UserDeposit;

#[derive(Accounts)]
pub struct CreateUserSplDeposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"lending_market"],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,
    pub mint: Account<'info, Mint>,

    #[account(
        seeds = [b"bank", mint.key().as_ref(), market.key().as_ref()],
        bump = bank.bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        init,
        payer = user,
        space = 8 + UserDeposit::INIT_SPACE,
        seeds = [b"user_deposit", market.key().as_ref(), bank.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    pub system_program: Program<'info, System>,
}


pub fn process_create_user_spl_deposit(
    ctx: Context<CreateUserSplDeposit>,
) -> Result<()> {
    let user_deposit = &mut ctx.accounts.user_deposit;

    user_deposit.user = ctx.accounts.user.key();
    user_deposit.mint = ctx.accounts.mint.key();
    user_deposit.amount = 0;
    user_deposit.bump = ctx.bumps.user_deposit;

    msg!(
        "User deposit account created for user {}",
        user_deposit.user
    );
    Ok(())
}