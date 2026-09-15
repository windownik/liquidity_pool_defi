use crate::state::bank::Bank;
use crate::structures::error::BankErrorCodes;
use crate::structures::market::Market;
use crate::structures::user_deposit::UserDeposit;
use anchor_lang::prelude::*;

use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_lang::solana_program::system_instruction;

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(
        seeds = [b"lending_market"],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"bank_solana_native", market.key().as_ref(),],
        bump,
    )]
    pub bank: Account<'info, Bank>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_deposit: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn process_deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
    require!(amount > 0, BankErrorCodes::InvalidAmount);
    let market = &ctx.accounts.market;
    require!(!market.is_paused, BankErrorCodes::InvalidAmount);

    let bank = &mut ctx.accounts.bank;
    let system_program = &ctx.accounts.system_program;

    let bump = create_user_deposit_if_needed(
        market.key(),
        &ctx.accounts.user,
        ctx.program_id,
        system_program,
        &mut ctx.accounts.user_deposit,
    )?;
    let mut user_deposit = Account::<UserDeposit>::try_from(
        &ctx.accounts.user_deposit.to_account_info()
    )?;
    // Transfer sol
    let ix = system_instruction::transfer(&ctx.accounts.user.key(), &bank.key(), amount);
    invoke(
        &ix,
        &[
            ctx.accounts.user_deposit.to_account_info(),
            bank.to_account_info(),
            system_program.to_account_info(),
        ],
    )?;

    if user_deposit.user == Pubkey::default() {
        user_deposit.user = ctx.accounts.user.key();
        user_deposit.mint = bank.mint;
        user_deposit.bump = bump;
        user_deposit.amount = amount;
    } else {
        user_deposit.amount = user_deposit
            .amount
            .checked_add(amount)
            .ok_or(BankErrorCodes::MathOverflow)?;
    }

    bank.total_deposits = bank
        .total_deposits
        .checked_add(amount)
        .ok_or(BankErrorCodes::MathOverflow)?;

    msg!("Successfully deposited {} lamports of Native SOL", amount);
    Ok(())
}

fn create_user_deposit_if_needed<'info>(
    market_key: Pubkey,
    user_account: & Signer<'info>,
    program_id: &Pubkey,
    system_program: & Program<'info, System>,
    user_deposit_info: &mut UncheckedAccount<'info>,
) -> Result<u8> {

    let (expected_pda, user_deposit_bump) = Pubkey::find_program_address(
        &[
            b"user_deposit",
            market_key.as_ref(),
            user_account.key().as_ref(),
        ],
        program_id,
    );
    require_keys_eq!(
        user_deposit_info.key(),
        expected_pda,
        BankErrorCodes::Unauthorized
    );
    if user_deposit_info.data_is_empty() {
        let space = 8 + UserDeposit::INIT_SPACE;
        let rent = Rent::get()?;
        let user_key = user_account.key();
        let bump_seed = [user_deposit_bump];

        let lamports = rent.minimum_balance(space);

        let create_acc_ix = system_instruction::create_account(
            &user_key,
            &user_deposit_info.key(),
            lamports,
            space as u64,
            program_id,
        );
        let signer_seeds: &[&[u8]] = &[
            b"user_deposit",
            market_key.as_ref(),
            user_key.as_ref(),
            &bump_seed,
        ];
        invoke_signed(
            &create_acc_ix,
            &[
                user_account.to_account_info(),
                user_deposit_info.to_account_info(),
                system_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;
        {
            let mut data = user_deposit_info.try_borrow_mut_data()?;
            let discriminator = UserDeposit::DISCRIMINATOR;
            data[..8].copy_from_slice(&discriminator);
        }
    }
    Ok(user_deposit_bump)
}
