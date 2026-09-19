
use anchor_lang::prelude::*;
use crate::structures::market::Market;
use crate::structures::error::BankErrorCodes;



#[derive(Accounts)]
pub struct UpdateIsPaused<'info> {
    #[account(
        mut,
        seeds = [b"lending_market"],
        bump = market.bump,
        has_one = admin @ BankErrorCodes::Unauthorized
    )]
    pub market: Account<'info, Market>,
    pub admin: Signer<'info>,
}


pub fn update_is_paused(
    ctx: Context<UpdateIsPaused>,
    is_paused: bool,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    // Обновляем статус
    market.is_paused = is_paused;

    msg!("Global market pause status updated to: {}", is_paused);

    Ok(())
}