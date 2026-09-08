# Anchor DeFi Lending Protocol — `lib.rs` Structure

---

### 1. Административная настройка рынка (Administrative & Initialization)

* **`initialize_market(ctx: Context<InitializeMarket>) -> Result<()>`**  
  Инициализация глобального кредитного рынка (Lending Market). Создает главный PDA-аккаунт рынка, назначает мультиподпись (`Multisig Authority`) и устанавливает базовые комиссии протокола.

* **`add_reserve(ctx: Context<AddReserve>, liquidation_threshold: u16, ltv: u16, liquidation_bonus: u16) -> Result<()>`**  
  Добавление нового токена (Reserve/Vault) на рынок (например, SOL, USDC). Создает PDA-сейф для физического хранения токенов (`bank_vault`) и конфиг риска: задает LTV, порог ликвидации и модель процентных ставок.

* **`update_reserve_config(ctx: Context<UpdateReserveConfig>, new_config: ReserveConfig) -> Result<()>`**  
  Обновление параметров риска для конкретного резерва под подписью Multisig. Позволяет менять LTV или замораживать операции с активом при высоком риске на рынке.

---

### 2. Базовые пользовательские операции (User Interactions)

* **`deposit(ctx: Context<Deposit>, amount: u64) -> Result<()>`**  
  Внесение депозита (Lending). Переводит токены пользователя на PDA `bank_vault` через CPI в Token Program, рассчитывает и зачисляет пропорциональную долю пула, включает актив как залог (`Collateral`).

* **`withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()>`**  
  Изъятие депозита из пула. Проверяет текущий `Health Factor` пользователя, чтобы изъятие залога не привело к дефолту позиции, после чего переводит токены из PDA обратно на кошелек пользователя.

* **`borrow(ctx: Context<Borrow>, amount: u64) -> Result<()>`**  
  Взятие займа под залог (Borrowing). Опрашивает оракулы (Pyth/Switchboard) для оценки текущих цен, вычисляет доступный лимит по LTV, увеличивает запись долга в `UserPosition` и переводит токены пользователю из `bank_vault`.

* **`repay(ctx: Context<Repay>, amount: u64) -> Result<()>`**  
  Погашение собственного долга (Repay). Переводит токены с кошелька заемщика обратно в PDA `bank_vault`, списывая тело долга и накопленные проценты из структуры `UserPosition`.

---

### 3. Движок ликвидации и актуализация данных (Engine & Cranks)

* **`accrue_interest(ctx: Context<AccrueInterest>) -> Result<()>`**  
  Перерасчет процентных ставок (Interest Accrual Crank). Вызывается перед любой операцией или отдельным ботом. Обновляет `cumulative_borrow_rate` на основе времени, прошедшего с прошлого вызова, и текущей утилизации пула (`Utilization Rate`).

* **`liquidate(ctx: Context<Liquidate>, max_liquidation_amount: u64) -> Result<()>`**  
  Бессерверная ликвидация "плохого" долга (Liquidation Engine). Запрашивает свежие цены с оракулов, проверяет `Health Factor < 1.0`, позволяет ликвидатору погасить часть долга жертвы и забрать залог заемщика с дисконтом (`Liquidation Bonus`).

---

### 4. Награды и миграция данных (Rewards & State Evolution)

* **`claim_rewards(ctx: Context<ClaimRewards>) -> Result<()>`**  
  Сбор наград за предоставление ликвидности (Claim Liquidity Mining Rewards). Принимает список PDA-аккаунтов эпох через `remaining_accounts`, рассчитывает накопленные токены управления и переводит их пользователю.

* **`migrate_user_position(ctx: Context<MigrateUserPosition>) -> Result<()>`**  
  Инструкция бесшовной миграции структуры данных (In-Place State Migration). Позволяет изменить структуру аккаунта `UserPosition` (например, через `realloc` с изменением версии структуры в байтах) без смены Program ID.