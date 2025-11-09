# Risk Management Guide

## Monitoring Loop

The `PositionMonitor` task evaluates each cached position every second:

1. Pulls the latest mark price (placeholder logic uses entry price).
2. Computes unrealized PnL and margin ratio using deterministic formulas.
3. Emits `liquidation_alert` events when margin ratio falls below `5%` (configurable).
4. Persists alerts to PostgreSQL and broadcasts to WebSocket subscribers.

## Liquidation Workflow

* Alerts are consumed by the liquidation engine which submits `close_position` transactions.
* Backend updates caches once the close instruction is confirmed.
* Position state transitions to `closed` and realized PnL is stored in history tables.

## Margin Management

* Leverage tiers cap notional exposure based on requested leverage and size.
* Initial margin locked on-chain equals `notional / leverage`.
* Maintenance margin equals initial margin × maintenance rate.
* Removing margin is forbidden when it would violate maintenance requirements.

## Funding Payments

* Funding accruals are tracked on-chain (`funding_accrued`).
* During close, backend subtracts the funding payment from gross PnL.

## Failure Handling

* Backend operations are idempotent—if Solana transactions fail, retry logic should resubmit.
* Database writes happen in transactions to prevent partial updates.
* Alerts are broadcast best-effort and should be acknowledged by downstream systems.
