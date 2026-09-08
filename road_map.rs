use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

// ============================================================================
// 1. АДМИНИСТРИРОВАНИЕ И НАСТРОЙКА РЫНКА (ADMINISTRATIVE & INITIALIZATION)
// ============================================================================

pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {
  let market = &mut ctx.accounts.market;

  market.admin = ctx.accounts.admin.key();
  market.bump = ctx.bumps.market;
  market.reserve_count = 0;

  msg!("Market initialized by admin: {}", market.admin);
  Ok(())
}

pub fn add_reserve(
  ctx: Context<AddReserve>,
  liquidation_threshold: u16,
  ltv: u16,
  liquidation_bonus: u16,
) -> Result<()> {
  require!(ltv <= liquidation_threshold, LendingError::InvalidRiskConfig);
  require!(liquidation_threshold <= 10000, LendingError::InvalidRiskConfig);

  let reserve = &mut ctx.accounts.reserve;
  let market = &mut ctx.accounts.market;

  reserve.market = market.key();
  reserve.mint = ctx.accounts.mint.key();
  reserve.vault = ctx.accounts.bank_vault.key();
  reserve.liquidation_threshold = liquidation_threshold;
  reserve.ltv = ltv;
  reserve.liquidation_bonus = liquidation_bonus;
  reserve.total_deposits = 0;
  reserve.total_borrows = 0;
  reserve.cumulative_borrow_rate = 1_000_000_000; // 1.0 in Wad format
  reserve.last_update_timestamp = Clock::get()?.unix_timestamp;
  reserve.bump = ctx.bumps.reserve;

  market.reserve_count += 1;

  msg!("Reserve added for mint: {}", reserve.mint);
  Ok(())
}

pub fn update_reserve_config(
  ctx: Context<UpdateReserveConfig>,
  new_config: ReserveConfig,
) -> Result<()> {
  require!(new_config.ltv <= new_config.liquidation_threshold, LendingError::InvalidRiskConfig);

  let reserve = &mut ctx.accounts.reserve;
  reserve.ltv = new_config.ltv;
  reserve.liquidation_threshold = new_config.liquidation_threshold;
  reserve.liquidation_bonus = new_config.liquidation_bonus;

  msg!("Reserve config updated for: {}", reserve.key());
  Ok(())
}


// ============================================================================
// 2. БАЗОВЫЕ ПОЛЬЗОВАТЕЛЬСКИЕ ОПЕРАЦИИ (DEPOSIT, WITHDRAW, BORROW, REPAY)
// ============================================================================

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
  require!(amount > 0, LendingError::InvalidAmount);

  // 1. CPI: Перевод токенов с кошелька пользователя на PDA bank_vault
  let cpi_accounts = Transfer {
    from: ctx.accounts.user_token_account.to_account_info(),
    to: ctx.accounts.bank_vault.to_account_info(),
    authority: ctx.accounts.user.to_account_info(),
  };
  let cpi_program = ctx.accounts.token_program.to_account_info();
  let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
  token::transfer(cpi_ctx, amount)?;

  // 2. Обновление состояния резерва и позиции пользователя
  let reserve = &mut ctx.accounts.reserve;
  let user_position = &mut ctx.accounts.user_position;

  reserve.total_deposits = reserve.total_deposits.checked_add(amount).ok_or(LendingError::MathOverflow)?;
  user_position.deposited_amount = user_position.deposited_amount.checked_add(amount).ok_or(LendingError::MathOverflow)?;

  msg!("Deposited {} tokens. New position balance: {}", amount, user_position.deposited_amount);
  Ok(())
}

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
  require!(amount > 0, LendingError::InvalidAmount);

  let user_position = &mut ctx.accounts.user_position;
  require!(user_position.deposited_amount >= amount, LendingError::InsufficientCollateral);

  // 1. Проверка Health Factor (убеждаемся, что снятие не вызовет недопокрытие долга)
  let remaining_collateral = user_position.deposited_amount.checked_sub(amount).ok_or(LendingError::MathOverflow)?;
  let health_factor = calculate_health_factor(remaining_collateral, user_position.borrowed_amount, &ctx.accounts.reserve)?;
  require!(health_factor >= 1_000_000_000, LendingError::HealthFactorTooLow);

  // 2. CPI: Перевод токенов из PDA bank_vault пользователю с использованием PDA-подписи
  let reserve_key = ctx.accounts.reserve.key();
  let seeds = &[
    b"bank_vault",
    reserve_key.as_ref(),
    &[ctx.accounts.reserve.vault_bump],
  ];
  let signer_seeds = &[&seeds[..]];

  let cpi_accounts = Transfer {
    from: ctx.accounts.bank_vault.to_account_info(),
    to: ctx.accounts.user_token_account.to_account_info(),
    authority: ctx.accounts.bank_vault.to_account_info(),
  };
  let cpi_program = ctx.accounts.token_program.to_account_info();
  let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
  token::transfer(cpi_ctx, amount)?;

  // Обновление состояния
  let reserve = &mut ctx.accounts.reserve;
  reserve.total_deposits = reserve.total_deposits.checked_sub(amount).ok_or(LendingError::MathOverflow)?;
  user_position.deposited_amount = remaining_collateral;

  msg!("Withdrawn {} tokens. Remaining deposit: {}", amount, user_position.deposited_amount);
  Ok(())
}

pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
  require!(amount > 0, LendingError::InvalidAmount);

  let reserve = &mut ctx.accounts.reserve;
  let user_position = &mut ctx.accounts.user_position;

  // 1. Проверка лимита по LTV (Максимальный доступный заем)
  let max_borrow = (user_position.deposited_amount as u128)
      .checked_mul(reserve.ltv as u128).ok_or(LendingError::MathOverflow)?
      .checked_div(10000).ok_or(LendingError::MathOverflow)? as u64;

  let new_borrowed_amount = user_position.borrowed_amount.checked_add(amount).ok_or(LendingError::MathOverflow)?;
  require!(new_borrowed_amount <= max_borrow, LendingError::ExceedsLtvLimit);

  // 2. Обновление состояния
  user_position.borrowed_amount = new_borrowed_amount;
  reserve.total_borrows = reserve.total_borrows.checked_add(amount).ok_or(LendingError::MathOverflow)?;

  // 3. CPI: Перевод запрашиваемых средств с PDA bank_vault на кошелек пользователя
  let reserve_key = reserve.key();
  let seeds = &[
    b"bank_vault",
    reserve_key.as_ref(),
    &[reserve.vault_bump],
  ];
  let signer_seeds = &[&seeds[..]];

  let cpi_accounts = Transfer {
    from: ctx.accounts.bank_vault.to_account_info(),
    to: ctx.accounts.user_token_account.to_account_info(),
    authority: ctx.accounts.bank_vault.to_account_info(),
  };
  let cpi_program = ctx.accounts.token_program.to_account_info();
  let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
  token::transfer(cpi_ctx, amount)?;

  msg!("Borrowed {} tokens. Total user borrow: {}", amount, user_position.borrowed_amount);
  Ok(())
}

pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
  require!(amount > 0, LendingError::InvalidAmount);

  let user_position = &mut ctx.accounts.user_position;
  let repay_amount = amount.min(user_position.borrowed_amount);

  // 1. CPI: Перевод токенов пользователя в bank_vault
  let cpi_accounts = Transfer {
    from: ctx.accounts.user_token_account.to_account_info(),
    to: ctx.accounts.bank_vault.to_account_info(),
    authority: ctx.accounts.user.to_account_info(),
  };
  let cpi_program = ctx.accounts.token_program.to_account_info();
  let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
  token::transfer(cpi_ctx, repay_amount)?;

  // 2. Списание долга
  let reserve = &mut ctx.accounts.reserve;
  user_position.borrowed_amount = user_position.borrowed_amount.checked_sub(repay_amount).ok_or(LendingError::MathOverflow)?;
  reserve.total_borrows = reserve.total_borrows.checked_sub(repay_amount).ok_or(LendingError::MathOverflow)?;

  msg!("Repaid {} tokens. Remaining debt: {}", repay_amount, user_position.borrowed_amount);
  Ok(())
}


// ============================================================================
// 3. ДВИЖОК ЛИКВИДАЦИИ И АКТУАЛИЗАЦИЯ ДАННЫХ (LIQUIDATIONS & CRANKS)
// ============================================================================

pub fn accrue_interest(ctx: Context<AccrueInterest>) -> Result<()> {
  let reserve = &mut ctx.accounts.reserve;
  let current_timestamp = Clock::get()?.unix_timestamp;
  let time_delta = current_timestamp.saturating_sub(reserve.last_update_timestamp);

  if time_delta > 0 && reserve.total_borrows > 0 {
    // Упрощенная модель плавающей ставки на основе Utilization Rate
    let utilization = (reserve.total_borrows as u128)
        .checked_mul(1_000_000_000).ok_or(LendingError::MathOverflow)?
        .checked_div(reserve.total_deposits as u128).unwrap_or(0);

    let base_rate = 20_000_000; // 2%
    let borrow_rate = base_rate.checked_add(utilization.checked_div(10).unwrap_or(0)).unwrap();

    let interest_factor = (borrow_rate as u128)
        .checked_mul(time_delta as u128).ok_or(LendingError::MathOverflow)?
        .checked_div(31_536_000).ok_or(LendingError::MathOverflow)?; // секунды в году

    reserve.cumulative_borrow_rate = reserve.cumulative_borrow_rate
        .checked_add(interest_factor as u64).ok_or(LendingError::MathOverflow)?;
    reserve.last_update_timestamp = current_timestamp;
  }

  Ok(())
}

pub fn liquidate(
  ctx: Context<Liquidate>,
  max_liquidation_amount: u64,
) -> Result<()> {
  let user_position = &mut ctx.accounts.borrower_position;
  let reserve = &ctx.accounts.reserve;

  // 1. Проверка Health Factor заемщика (разрешено при HF < 1.0)
  let health_factor = calculate_health_factor(
    user_position.deposited_amount,
    user_position.borrowed_amount,
    reserve,
  )?;
  require!(health_factor < 1_000_000_000, LendingError::PositionNotLiquidatable);

  // 2. Расчет ликвидируемой суммы долга
  let actual_repay = max_liquidation_amount.min(user_position.borrowed_amount);

  // 3. Вычисление залога к изъятию с учетом Liquidation Bonus (например +5%)
  let collateral_to_seize = (actual_repay as u128)
      .checked_mul(10000 + reserve.liquidation_bonus as u128).ok_or(LendingError::MathOverflow)?
      .checked_div(10000).ok_or(LendingError::MathOverflow)? as u64;

  require!(user_position.deposited_amount >= collateral_to_seize, LendingError::InsufficientCollateralToSeize);

  // 4. CPI: Ликвидатор гасит долг заемщика
  let cpi_repay_accounts = Transfer {
    from: ctx.accounts.liquidator_token_account.to_account_info(),
    to: ctx.accounts.bank_vault.to_account_info(),
    authority: ctx.accounts.liquidator.to_account_info(),
  };
  let cpi_program = ctx.accounts.token_program.to_account_info();
  token::transfer(CpiContext::new(cpi_program.clone(), cpi_repay_accounts), actual_repay)?;

  // 5. CPI: Перевод залога заемщика на кошелек ликвидатора с подписью PDA
  let reserve_key = reserve.key();
  let seeds = &[
    b"bank_vault",
    reserve_key.as_ref(),
    &[reserve.vault_bump],
  ];
  let signer_seeds = &[&seeds[..]];

  let cpi_seize_accounts = Transfer {
    from: ctx.accounts.bank_vault.to_account_info(),
    to: ctx.accounts.liquidator_collateral_account.to_account_info(),
    authority: ctx.accounts.bank_vault.to_account_info(),
  };
  token::transfer(
    CpiContext::new_with_signer(cpi_program, cpi_seize_accounts, signer_seeds),
    collateral_to_seize,
  )?;

  // 6. Обновление позиции жертвы
  user_position.borrowed_amount = user_position.borrowed_amount.checked_sub(actual_repay).ok_or(LendingError::MathOverflow)?;
  user_position.deposited_amount = user_position.deposited_amount.checked_sub(collateral_to_seize).ok_or(LendingError::MathOverflow)?;

  msg!("Liquidation executed! Repaid: {}, Seized collateral: {}", actual_repay, collateral_to_seize);
  Ok(())
}


// ============================================================================
// 4. НАГРАДЫ, СТЕЙКИНГ И ЭВОЛЮЦИЯ СТРУКТУР (REWARDS & MIGRATION)
// ============================================================================

pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
  let mut total_reward: u64 = 0;

  // Считывание эпох наград через remaining_accounts
  for epoch_info in ctx.remaining_accounts.iter() {
    let reward_epoch = Account::<RewardEpoch>::try_from(epoch_info)?;
    if !reward_epoch.is_claimed {
      total_reward = total_reward.checked_add(reward_epoch.amount).ok_or(LendingError::MathOverflow)?;
    }
  }

  require!(total_reward > 0, LendingError::NoRewardsAvailable);

  // CPI: Перевод токенов наград с PDA pool пользователя
  let seeds = &[b"reward_vault", &[ctx.bumps.reward_vault]];
  let signer_seeds = &[&seeds[..]];

  let cpi_accounts = Transfer {
    from: ctx.accounts.reward_vault.to_account_info(),
    to: ctx.accounts.user_reward_account.to_account_info(),
    authority: ctx.accounts.reward_vault.to_account_info(),
  };
  let cpi_ctx = CpiContext::new_with_signer(
    ctx.accounts.token_program.to_account_info(),
    cpi_accounts,
    signer_seeds,
  );
  token::transfer(cpi_ctx, total_reward)?;

  msg!("Claimed total rewards: {}", total_reward);
  Ok(())
}

pub fn migrate_user_position(ctx: Context<MigrateUserPosition>) -> Result<()> {
  let user_position = &mut ctx.accounts.user_position;

  // Проверка версии перед миграцией
  require!(user_position.version < TARGET_POSITION_VERSION, LendingError::AlreadyMigrated);

  // Обновление версии структуры
  user_position.version = TARGET_POSITION_VERSION;

  msg!("User position migrated to version: {}", TARGET_POSITION_VERSION);
  Ok(())
}


// ============================================================================
// ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ И ОШИБКИ
// ============================================================================

fn calculate_health_factor(collateral: u64, borrowed: u64, reserve: &Account<Reserve>) -> Result<u64> {
  if borrowed == 0 {
    return Ok(u64::MAX); // Если нет долга, позиция максимально здорова
  }

  let adjusted_collateral = (collateral as u128)
      .checked_mul(reserve.liquidation_threshold as u128).ok_or(LendingError::MathOverflow)?
      .checked_div(10000).ok_or(LendingError::MathOverflow)?;

  let health_factor = adjusted_collateral
      .checked_mul(1_000_000_000).ok_or(LendingError::MathOverflow)?
      .checked_div(borrowed as u128).ok_or(LendingError::MathOverflow)?;

  Ok(health_factor as u64)
}

const TARGET_POSITION_VERSION: u8 = 2;

#[error_code]
pub enum LendingError {
  #[msg("Invalid risk configuration provided")]
  InvalidRiskConfig,
  #[msg("Amount must be greater than zero")]
  InvalidAmount,
  #[msg("Math calculation overflow")]
  MathOverflow,
  #[msg("Insufficient collateral for this action")]
  InsufficientCollateral,
  #[msg("Health factor too low to perform withdraw")]
  HealthFactorTooLow,
  #[msg("Requested borrow amount exceeds LTV limit")]
  ExceedsLtvLimit,
  #[msg("Position is healthy and cannot be liquidated")]
  PositionNotLiquidatable,
  #[msg("Insufficient collateral to cover liquidation bonus")]
  InsufficientCollateralToSeize,
  #[msg("No rewards available for claim")]
  NoRewardsAvailable,
  #[msg("Account already migrated to latest version")]
  AlreadyMigrated,
}