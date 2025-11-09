# Testing Strategy

## On-Chain Tests

* Anchor program unit tests should cover:
  * Opening positions across leverage tiers
  * Margin reduction safeguards
  * Liquidation price correctness for long/short positions
  * PnL settlement with funding adjustments
* Property tests (via `proptest`) target overflow scenarios in fixed-point math.

## Backend Tests

* Unit tests exist for margin and liquidation helper functions (`backend/src/margin.rs`).
* Integration tests should mock Solana RPC and PostgreSQL to validate the REST API contract.
* Fuzz tests can target JSON payloads for request validation.

## Load Testing

* Benchmark API endpoints using `k6` or `wrk` to validate 100+ position ops/sec.
* Stress WebSocket broadcasting to ensure 10k concurrent clients remain under 100ms update latency.

## Continuous Integration

A suggested CI pipeline:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `anchor test`
5. Database migration check with `sqlx migrate run --check` (if sqlx is adopted)
