use anchor_lang::prelude::*;

pub const MAX_SYMBOL_LEN: usize = 16;
pub const RATE_SCALE: u128 = 1_000_000; // 1e6 precision

declare_id!("PosMgnmt111111111111111111111111111111111");

#[program]
pub mod position_manager {
    use super::*;

    pub fn open_position(
        ctx: Context<OpenPosition>,
        symbol: String,
        side: Side,
        size: u64,
        leverage: u16,
        entry_price: u64,
    ) -> Result<()> {
        require!(!symbol.is_empty(), PositionError::InvalidSymbol);
        require!(symbol.len() <= MAX_SYMBOL_LEN, PositionError::InvalidSymbol);
        require!((1..=1_000).contains(&leverage), PositionError::InvalidLeverage);
        require!(size > 0, PositionError::InvalidPositionSize);

        let tier = get_leverage_tier(leverage, size)?;

        let initial_margin = calculate_initial_margin(size, entry_price, leverage)?;
        let maintenance_margin = calculate_maintenance_margin(size, entry_price, tier.maintenance_margin_rate)?;
        require!(initial_margin > 0, PositionError::InvalidMarginAmount);
        require!(maintenance_margin > 0, PositionError::InvalidMarginAmount);

        let user = &mut ctx.accounts.user_account;
        if user.position_count == 0 {
            user.owner = ctx.accounts.owner.key();
            user.bump = *ctx.bumps.get("user_account").ok_or(PositionError::BumpNotFound)?;
        } else {
            require_keys_eq!(user.owner, ctx.accounts.owner.key(), PositionError::Unauthorized);
        }

        let available_collateral = user
            .total_collateral
            .checked_sub(user.locked_collateral)
            .ok_or(PositionError::MathOverflow)?;
        require!(available_collateral >= initial_margin, PositionError::InsufficientCollateral);

        user.locked_collateral = user
            .locked_collateral
            .checked_add(initial_margin)
            .ok_or(PositionError::MathOverflow)?;
        user.position_count = user
            .position_count
            .checked_add(1)
            .ok_or(PositionError::MathOverflow)?;

        let position = &mut ctx.accounts.position;
        position.owner = ctx.accounts.owner.key();
        position.symbol = symbol.clone();
        position.side = side;
        position.size = size;
        position.entry_price = entry_price;
        position.margin = initial_margin;
        position.leverage = leverage as u8;
        position.unrealized_pnl = 0;
        position.realized_pnl = 0;
        position.funding_accrued = 0;
        position.liquidation_price = calculate_liquidation_price(entry_price, side, leverage, tier.maintenance_margin_rate)?;
        position.maintenance_margin = maintenance_margin;
        position.bump = *ctx.bumps.get("position").ok_or(PositionError::BumpNotFound)?;
        position.last_update = Clock::get()?.unix_timestamp;

        emit!(PositionOpened {
            owner: ctx.accounts.owner.key(),
            symbol,
            side,
            size,
            leverage,
            entry_price,
            margin: initial_margin,
        });

        Ok(())
    }

    pub fn modify_position(
        ctx: Context<ModifyPosition>,
        size_delta: i64,
        margin_delta: i64,
        new_entry_price: Option<u64>,
        new_leverage: Option<u16>,
    ) -> Result<()> {
        let position = &mut ctx.accounts.position;
        let user = &mut ctx.accounts.user_account;
        require_keys_eq!(position.owner, ctx.accounts.owner.key(), PositionError::Unauthorized);

        if let Some(price) = new_entry_price {
            require!(price > 0, PositionError::InvalidPrice);
            position.entry_price = price;
        }

        if let Some(lev) = new_leverage {
            require!((1..=1_000).contains(&lev), PositionError::InvalidLeverage);
            let tier = get_leverage_tier(lev, position.size)?;
            position.leverage = lev as u8;
            position.liquidation_price = calculate_liquidation_price(
                position.entry_price,
                position.side,
                lev,
                tier.maintenance_margin_rate,
            )?;
            position.maintenance_margin = calculate_maintenance_margin(
                position.size,
                position.entry_price,
                tier.maintenance_margin_rate,
            )?;
        }

        if size_delta != 0 {
            let new_size = if size_delta.is_positive() {
                position
                    .size
                    .checked_add(size_delta as u64)
                    .ok_or(PositionError::MathOverflow)?
            } else {
                let delta = size_delta.unsigned_abs();
                position
                    .size
                    .checked_sub(delta)
                    .ok_or(PositionError::InvalidPositionSize)?
            };
            require!(new_size > 0, PositionError::InvalidPositionSize);

            let leverage = position.leverage as u16;
            let tier = get_leverage_tier(leverage, new_size)?;
            let new_maintenance = calculate_maintenance_margin(
                new_size,
                position.entry_price,
                tier.maintenance_margin_rate,
            )?;
            let new_margin = calculate_initial_margin(new_size, position.entry_price, leverage)?;
            let old_margin = position.margin;
            if new_margin > old_margin {
                let diff = new_margin
                    .checked_sub(old_margin)
                    .ok_or(PositionError::MathOverflow)?;
                let available = user
                    .total_collateral
                    .checked_sub(user.locked_collateral)
                    .ok_or(PositionError::MathOverflow)?;
                require!(available >= diff, PositionError::InsufficientCollateral);
                user.locked_collateral = user
                    .locked_collateral
                    .checked_add(diff)
                    .ok_or(PositionError::MathOverflow)?;
            } else if old_margin > new_margin {
                let diff = old_margin
                    .checked_sub(new_margin)
                    .ok_or(PositionError::MathOverflow)?;
                require!(user.locked_collateral >= diff, PositionError::InvalidMarginAmount);
                user.locked_collateral = user
                    .locked_collateral
                    .checked_sub(diff)
                    .ok_or(PositionError::MathOverflow)?;
            }
            position.size = new_size;
            position.margin = new_margin;
            position.maintenance_margin = new_maintenance;
            position.liquidation_price = calculate_liquidation_price(
                position.entry_price,
                position.side,
                leverage,
                tier.maintenance_margin_rate,
            )?;
        }

        if margin_delta != 0 {
            if margin_delta.is_positive() {
                let delta = margin_delta as u64;
                let available_collateral = user
                    .total_collateral
                    .checked_sub(user.locked_collateral)
                    .ok_or(PositionError::MathOverflow)?;
                require!(available_collateral >= delta, PositionError::InsufficientCollateral);
                user.locked_collateral = user
                    .locked_collateral
                    .checked_add(delta)
                    .ok_or(PositionError::MathOverflow)?;
                position.margin = position
                    .margin
                    .checked_add(delta)
                    .ok_or(PositionError::MathOverflow)?;
            } else {
                let delta = margin_delta.unsigned_abs();
                require!(position.margin >= delta, PositionError::InvalidMarginAmount);
                require!(user.locked_collateral >= delta, PositionError::InvalidMarginAmount);
                position.margin = position
                    .margin
                    .checked_sub(delta)
                    .ok_or(PositionError::MathOverflow)?;
                user.locked_collateral = user
                    .locked_collateral
                    .checked_sub(delta)
                    .ok_or(PositionError::MathOverflow)?;
            }
        }

        position.last_update = Clock::get()?.unix_timestamp;

        emit!(PositionModified {
            owner: position.owner,
            position: ctx.accounts.position.key(),
            size: position.size,
            margin: position.margin,
            leverage: position.leverage,
        });

        Ok(())
    }

    pub fn close_position(
        ctx: Context<ClosePosition>,
        exit_price: u64,
        funding_payment: i64,
    ) -> Result<()> {
        require!(exit_price > 0, PositionError::InvalidPrice);
        let position = &mut ctx.accounts.position;
        let user = &mut ctx.accounts.user_account;
        require_keys_eq!(position.owner, ctx.accounts.owner.key(), PositionError::Unauthorized);

        let pnl = calculate_pnl(position.side, position.size, position.entry_price, exit_price)?;
        let net_pnl = pnl
            .checked_sub(funding_payment)
            .ok_or(PositionError::MathOverflow)?;

        let margin = position.margin;
        require!(user.locked_collateral >= margin, PositionError::InvalidMarginAmount);

        user.locked_collateral = user
            .locked_collateral
            .checked_sub(margin)
            .ok_or(PositionError::MathOverflow)?;
        user.total_pnl = user
            .total_pnl
            .checked_add(net_pnl)
            .ok_or(PositionError::MathOverflow)?;

        if net_pnl >= 0 {
            user.total_collateral = user
                .total_collateral
                .checked_add(net_pnl as u64)
                .ok_or(PositionError::MathOverflow)?;
        } else {
            let loss = net_pnl.unsigned_abs();
            user.total_collateral = user
                .total_collateral
                .checked_sub(loss)
                .ok_or(PositionError::InsufficientCollateral)?;
        }

        user.position_count = user
            .position_count
            .checked_sub(1)
            .ok_or(PositionError::MathOverflow)?;

        emit!(PositionClosed {
            owner: position.owner,
            position: ctx.accounts.position.key(),
            exit_price,
            realized_pnl: net_pnl,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(symbol: String)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + UserAccount::LEN,
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        init,
        payer = owner,
        space = 8 + Position::LEN,
        seeds = [b"position", owner.key().as_ref(), symbol.as_bytes()],
        bump
    )]
    pub position: Account<'info, Position>,
    /// CHECK: Collateral vault is managed by the wider protocol
    #[account(mut)]
    pub collateral_vault: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(symbol: String)]
pub struct ModifyPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump = user_account.bump,
        has_one = owner
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        mut,
        seeds = [b"position", owner.key().as_ref(), symbol.as_bytes()],
        bump = position.bump,
        has_one = owner
    )]
    pub position: Account<'info, Position>,
}

#[derive(Accounts)]
#[instruction(symbol: String)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump = user_account.bump,
        has_one = owner
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        mut,
        close = owner,
        seeds = [b"position", owner.key().as_ref(), symbol.as_bytes()],
        bump = position.bump,
        has_one = owner
    )]
    pub position: Account<'info, Position>,
}

#[account]
pub struct Position {
    pub owner: Pubkey,
    #[max_len(MAX_SYMBOL_LEN)]
    pub symbol: String,
    pub side: Side,
    pub size: u64,
    pub entry_price: u64,
    pub margin: u64,
    pub leverage: u8,
    pub unrealized_pnl: i64,
    pub realized_pnl: i64,
    pub funding_accrued: i64,
    pub liquidation_price: u64,
    pub maintenance_margin: u64,
    pub last_update: i64,
    pub bump: u8,
}

impl Position {
    pub const LEN: usize = 32 + 4 + MAX_SYMBOL_LEN + 1 + 8 + 8 + 8 + 1 + 8 + 8 + 8 + 8 + 8 + 1;
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub total_collateral: u64,
    pub locked_collateral: u64,
    pub total_pnl: i64,
    pub position_count: u32,
    pub bump: u8,
}

impl UserAccount {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 4 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Long,
    Short,
}

impl Side {
    pub fn multiplier(&self) -> i64 {
        match self {
            Side::Long => 1,
            Side::Short => -1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LeverageTier {
    pub max_leverage: u16,
    pub initial_margin_rate: u64,
    pub maintenance_margin_rate: u64,
    pub max_position_size: u64,
}

pub const LEVERAGE_TIERS: [LeverageTier; 5] = [
    LeverageTier {
        max_leverage: 20,
        initial_margin_rate: 50_000,
        maintenance_margin_rate: 25_000,
        max_position_size: u64::MAX,
    },
    LeverageTier {
        max_leverage: 50,
        initial_margin_rate: 20_000,
        maintenance_margin_rate: 10_000,
        max_position_size: 100_000,
    },
    LeverageTier {
        max_leverage: 100,
        initial_margin_rate: 10_000,
        maintenance_margin_rate: 5_000,
        max_position_size: 50_000,
    },
    LeverageTier {
        max_leverage: 500,
        initial_margin_rate: 5_000,
        maintenance_margin_rate: 2_500,
        max_position_size: 20_000,
    },
    LeverageTier {
        max_leverage: 1_000,
        initial_margin_rate: 2_000,
        maintenance_margin_rate: 1_000,
        max_position_size: 5_000,
    },
];

fn get_leverage_tier(leverage: u16, position_size: u64) -> Result<LeverageTier> {
    for tier in LEVERAGE_TIERS.iter() {
        if leverage <= tier.max_leverage && position_size <= tier.max_position_size {
            return Ok(tier.clone());
        }
    }
    Err(error!(PositionError::LeverageExceeded))
}

fn calculate_initial_margin(size: u64, entry_price: u64, leverage: u16) -> Result<u64> {
    let position_value = (size as u128)
        .checked_mul(entry_price as u128)
        .ok_or(PositionError::MathOverflow)?;
    let leverage = leverage as u128;
    let margin = position_value
        .checked_div(leverage)
        .ok_or(PositionError::MathOverflow)?;
    Ok(margin as u64)
}

fn calculate_maintenance_margin(size: u64, entry_price: u64, maintenance_rate: u64) -> Result<u64> {
    let position_value = (size as u128)
        .checked_mul(entry_price as u128)
        .ok_or(PositionError::MathOverflow)?;
    let margin = position_value
        .checked_mul(maintenance_rate as u128)
        .ok_or(PositionError::MathOverflow)?
        .checked_div(RATE_SCALE)
        .ok_or(PositionError::MathOverflow)?;
    Ok(margin as u64)
}

fn calculate_liquidation_price(
    entry_price: u64,
    side: Side,
    leverage: u16,
    maintenance_margin_rate: u64,
) -> Result<u64> {
    let leverage_inv = RATE_SCALE
        .checked_div(leverage as u128)
        .ok_or(PositionError::MathOverflow)?;
    let base = match side {
        Side::Long => RATE_SCALE
            .checked_sub(leverage_inv)
            .ok_or(PositionError::MathOverflow)?
            .checked_add(maintenance_margin_rate as u128)
            .ok_or(PositionError::MathOverflow)?,
        Side::Short => RATE_SCALE
            .checked_add(leverage_inv)
            .ok_or(PositionError::MathOverflow)?
            .checked_sub(maintenance_margin_rate as u128)
            .ok_or(PositionError::MathOverflow)?,
    };

    let price = (entry_price as u128)
        .checked_mul(base)
        .ok_or(PositionError::MathOverflow)?
        .checked_div(RATE_SCALE)
        .ok_or(PositionError::MathOverflow)?;
    Ok(price as u64)
}

fn calculate_pnl(side: Side, size: u64, entry_price: u64, exit_price: u64) -> Result<i64> {
    let price_diff = match side {
        Side::Long => exit_price as i128 - entry_price as i128,
        Side::Short => entry_price as i128 - exit_price as i128,
    };
    let pnl = (size as i128)
        .checked_mul(price_diff)
        .ok_or(PositionError::MathOverflow)?;
    Ok(pnl as i64)
}

#[event]
pub struct PositionOpened {
    pub owner: Pubkey,
    pub symbol: String,
    pub side: Side,
    pub size: u64,
    pub leverage: u16,
    pub entry_price: u64,
    pub margin: u64,
}

#[event]
pub struct PositionModified {
    pub owner: Pubkey,
    pub position: Pubkey,
    pub size: u64,
    pub margin: u64,
    pub leverage: u8,
}

#[event]
pub struct PositionClosed {
    pub owner: Pubkey,
    pub position: Pubkey,
    pub exit_price: u64,
    pub realized_pnl: i64,
}

#[error_code]
pub enum PositionError {
    #[msg("Invalid trading symbol provided")]
    InvalidSymbol,
    #[msg("Invalid leverage value")]
    InvalidLeverage,
    #[msg("Invalid position size")]
    InvalidPositionSize,
    #[msg("Insufficient collateral to perform action")]
    InsufficientCollateral,
    #[msg("Mathematical overflow detected")]
    MathOverflow,
    #[msg("Unauthorized action for owner")]
    Unauthorized,
    #[msg("Invalid margin amount")]
    InvalidMarginAmount,
    #[msg("Leverage exceeds protocol limits")]
    LeverageExceeded,
    #[msg("Bump not found")]
    BumpNotFound,
    #[msg("Invalid price value")]
    InvalidPrice,
}

