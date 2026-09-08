use anchor_lang::prelude::*;
use crate::structures::market::Market;
use crate::structures::error::BankErrorCodes;



#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Market::INIT_SPACE,
        seeds = [b"lending_market"],
        bump,
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Кошелек для сбора комиссий
    pub fee_receiver: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

}


pub fn process_initialize_market(
    ctx: Context<InitializeMarket>,
    protocol_fee_bps: u16,
) -> Result<()> {
    require!(
        protocol_fee_bps <= 10_000,
        BankErrorCodes::InvalidProtocolFee
    );

    let market = &mut ctx.accounts.market;

    // Тот, кто вызвал эту функцию первыми, записывается как единственный admin
    market.admin = ctx.accounts.admin.key();
    market.fee_receiver = ctx.accounts.fee_receiver.key();
    market.reserves_count = 0;
    market.protocol_fee_bps = protocol_fee_bps;
    market.is_paused = false;
    market.version = 1;
    market.bump = ctx.bumps.market;

    msg!("Global Lending Market initialized. Admin: {}", market.admin);

    Ok(())
}