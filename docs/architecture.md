# Architecture Overview

The position management system is composed of two tightly coupled layers: an on-chain Anchor program and an off-chain Rust service.

## Components

### Anchor Program (`programs/position_manager`)

* Owns and maintains PDA-based position accounts keyed by `("position", owner, symbol)`.
* Stores user collateral state inside `UserAccount` PDA keyed by `("user", owner)`.
* Validates leverage tiers, initial/maintenance margin, and liquidation thresholds using deterministic fixed-point arithmetic.
* Emits events (`PositionOpened`, `PositionModified`, `PositionClosed`) that are consumed by the backend for analytics.

### Backend Service (`backend`)

* Provides REST APIs for CRUD operations on positions and user views.
* Streams WebSocket updates for real-time UI rendering and risk monitoring.
* Persists canonical position history in PostgreSQL using the schema under `backend/sql/schema.sql`.
* Monitors every cached position, recomputing margin ratios and generating liquidation alerts.
* Bridges to Solana via the Anchor client for transaction submission and account polling (stubbed in this reference implementation).

### Data Flow

1. Client submits a REST request (`/positions/open`).
2. Backend validates the payload, prepares Solana transaction payload, and queues the request.
3. Upon confirmation, on-chain PDAs are updated and events emitted.
4. Backend ingests events, writes historical rows to PostgreSQL, and broadcasts updates to WebSocket subscribers.
5. Position monitor periodically fetches mark prices from the oracle (mocked) and computes risk metrics, triggering liquidation alerts.

### State Machine

```
OPENING -> OPEN -> MODIFYING -> OPEN -> CLOSING -> CLOSED
                       |
                  LIQUIDATING
```

Each transition is triggered by a Solana instruction or an off-chain liquidation workflow.

## Deployment Topology

* **Solana Cluster** – Runs the Anchor program and holds PDAs.
* **Backend Service** – Stateless Axum server, horizontally scalable.
* **PostgreSQL** – Stores historical analytics and user metrics.
* **Liquidation Engine** – External service subscribing to WebSocket alerts and executing forced closures.

## Security Considerations

* PDA seeds are deterministic, preventing duplicate positions per owner/symbol pair.
* Margin arithmetic relies exclusively on integer math with a 1e6 fixed-point scale.
* Backend enforces caching and reconciliation to avoid mismatched state between Solana and the database.
