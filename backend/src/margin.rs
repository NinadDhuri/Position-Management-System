use rust_decimal::prelude::*;
use rust_decimal::Decimal;

use crate::errors::{PositionServiceError, Result};

pub fn calculate_average_entry_price(trades: &[(Decimal, Decimal)]) -> Result<Decimal> {
    let (total_value, total_size) = trades.iter().try_fold(
        (Decimal::ZERO, Decimal::ZERO),
        |(value_acc, size_acc), (price, qty)| {
            if qty.is_sign_negative() {
                return Err(PositionServiceError::Validation("Trade quantity must be positive".into()));
            }
            Ok((value_acc + (*price * *qty), size_acc + *qty))
        },
    )?;

    if total_size.is_zero() {
        return Err(PositionServiceError::Validation("Total trade size cannot be zero".into()));
    }

    Ok(total_value / total_size)
}

pub fn calculate_unrealized_pnl(
    is_long: bool,
    size: Decimal,
    mark_price: Decimal,
    entry_price: Decimal,
) -> Decimal {
    let price_diff = if is_long {
        mark_price - entry_price
    } else {
        entry_price - mark_price
    };
    size * price_diff
}

pub fn calculate_margin_ratio(
    collateral: Decimal,
    unrealized_pnl: Decimal,
    size: Decimal,
    mark_price: Decimal,
) -> Result<Decimal> {
    let position_value = size * mark_price;
    if position_value.is_zero() {
        return Err(PositionServiceError::Validation("Position value cannot be zero".into()));
    }
    Ok((collateral + unrealized_pnl) / position_value)
}

pub fn calculate_liquidation_price_long(
    entry_price: Decimal,
    leverage: Decimal,
    maintenance_margin_ratio: Decimal,
) -> Result<Decimal> {
    let leverage_inv = Decimal::ONE / leverage;
    Ok(entry_price * (Decimal::ONE - leverage_inv + maintenance_margin_ratio))
}

pub fn calculate_liquidation_price_short(
    entry_price: Decimal,
    leverage: Decimal,
    maintenance_margin_ratio: Decimal,
) -> Result<Decimal> {
    let leverage_inv = Decimal::ONE / leverage;
    Ok(entry_price * (Decimal::ONE + leverage_inv - maintenance_margin_ratio))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(value: f64) -> Decimal {
        Decimal::from_f64(value).unwrap()
    }

    #[test]
    fn test_average_entry_price() {
        let trades = vec![(d(100.0), d(1.0)), (d(110.0), d(2.0))];
        let price = calculate_average_entry_price(&trades).unwrap();
        assert_eq!(price.round_dp(2), d(106.67));
    }

    #[test]
    fn test_unrealized_pnl_long() {
        let pnl = calculate_unrealized_pnl(true, d(2.0), d(120.0), d(100.0));
        assert_eq!(pnl, d(40.0));
    }

    #[test]
    fn test_margin_ratio() {
        let ratio = calculate_margin_ratio(d(1000.0), d(100.0), d(2.0), d(500.0)).unwrap();
        assert_eq!(ratio.round_dp(4), d(0.55));
    }

    #[test]
    fn test_liquidation_long() {
        let price = calculate_liquidation_price_long(d(100.0), d(10.0), d(0.05)).unwrap();
        assert_eq!(price.round_dp(2), d(95.0));
    }

    #[test]
    fn test_liquidation_short() {
        let price = calculate_liquidation_price_short(d(100.0), d(10.0), d(0.05)).unwrap();
        assert_eq!(price.round_dp(2), d(105.0));
    }
}
