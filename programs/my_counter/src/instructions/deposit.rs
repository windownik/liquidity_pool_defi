use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::structures::market::Market;
use crate::structures::error::BankErrorCodes;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [b"lending_market"],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    // Аккаунт состояния токена в банке (хранит лимиты, суммы и ссылку на vault)
    #[account(
        mut,
        seeds = [b"bank", user.key(), mint],
        bump = bank.bump,
    )]
    pub bank: Account<'info, Market>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Испольуется только при депозите Native SOL для получения lamports
    #[account(
        mut,
        seeds = [b"sol_vault", market.key().as_ref()],
        bump
    )]
    pub sol_vault: UncheckedAccount<'info>,

    // Токеновый аккаунт пользователя (опционален / передается при SPL-депозитах)
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == bank.mint
    )]
    pub user_token_account: Option<Account<'info, TokenAccount>>,

    // Токеновый сейф банка для SPL-токенов
    #[account(
        mut,
        address = bank.vault
    )]
    pub bank_vault: Option<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Option<Program<'info, Token>>,
}


pub fn process_deposit(
    ctx: Context<Deposit>,
    amount: u64,
    currency: Currency
) -> Result<()> {
    require!(amount > 0, BankErrorCodes::InvalidAmount);

    let bank = &mut ctx.accounts.bank;

    // Проверка лимитов депозита (если заданы)
    if bank.deposit_limit > 0 {
        require!(
            bank.total_deposits.saturating_add(amount) <= bank.deposit_limit,
            BankErrorCodes::DepositLimitExceeded
        );
    }

    match currency {
        Currency::Sol => {
            // === ПОПОЛНЕНИЕ NATIVE SOL ===
            // Перевод Native SOL от Signer на PDA Vault через System Program
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.user.to_account_info(),
                    to: ctx.accounts.sol_vault.to_account_info(),
                },
            );
            system_program::transfer(cpi_context, amount)?;
        }
        Currency::Usdc | Currency::Custom { .. } => {
            // === ПОПОЛНЕНИЕ SPL TOKEN (USDC и др.) ===
            let user_ta = ctx.accounts.user_token_account.as_ref()
                .ok_or(BankErrorCodes::MissingTokenAccount)?;
            let bank_vault = ctx.accounts.bank_vault.as_ref()
                .ok_or(BankErrorCodes::MissingBankVault)?;
            let token_program = ctx.accounts.token_program.as_ref()
                .ok_or(BankErrorCodes::MissingTokenProgram)?;

            let cpi_accounts = Transfer {
                from: user_ta.to_account_info(),
                to: bank_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            };

            let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);
            token::transfer(cpi_ctx, amount)?;
        }
    }

    // === ОБНОВЛЕНИЕ СОСТОЯНИЯ ===
    bank.total_deposits = bank.total_deposits.checked_add(amount)
        .ok_or(BankErrorCodes::MathOverflow)?;

    // Если у вас есть аккаунт баланса конкретного пользователя (UserPosition / Obligation)
    // здесь нужно обновить его личный баланс:
    // user_position.deposited_amount += amount;

    msg!("Deposit successful: {} of {:?}", amount, currency);
    Ok(())
}