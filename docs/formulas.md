# Margin & PnL Formulas

All monetary calculations are implemented with deterministic fixed-point arithmetic.

## Fixed-Point Representation

* On-chain calculations use integers scaled by `1_000_000` (`RATE_SCALE`).
* Off-chain service relies on `rust_decimal::Decimal` to avoid floating point drift.

## Initial Margin

```
initial_margin = (position_size * entry_price) / leverage
```

## Maintenance Margin

```
maintenance_margin = (position_size * entry_price) * maintenance_margin_rate
```

`maintenance_margin_rate` is stored in 1e-6 precision.

## Margin Ratio

```
position_value = position_size * mark_price
margin_ratio = (margin + unrealized_pnl) / position_value
```

## Unrealized PnL

```
if side == Long:
    unrealized_pnl = position_size * (mark_price - entry_price)
else:
    unrealized_pnl = position_size * (entry_price - mark_price)
```

## Liquidation Price

For long positions:

```
liquidation_price = entry_price * (1 - 1/leverage + maintenance_margin_rate)
```

For short positions:

```
liquidation_price = entry_price * (1 + 1/leverage - maintenance_margin_rate)
```

## Funding Payments

Funding accrual is stored as signed integers and applied during position close:

```
realized_pnl = gross_pnl - funding_payment
```
