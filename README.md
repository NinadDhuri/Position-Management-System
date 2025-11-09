# Position Management System

This repository contains a reference implementation of a high leverage perpetual futures position management stack. It is split into two major components:

* An [Anchor](https://www.anchor-lang.com/) smart contract located at `programs/position_manager` that manages on-chain position state, margin accounting, and liquidation thresholds.
* A Rust backend service in `backend` that provides real-time monitoring, REST/WebSocket APIs, database persistence, and risk management utilities.

The project is designed for educational and prototyping purposes. It is **not** production ready without rigorous security audits, on-chain integration tests, and infrastructure hardening.

## Repository layout

```
.
├── Anchor.toml
├── Cargo.toml
├── README.md
├── backend
│   ├── Cargo.toml
│   ├── sql
│   │   └── schema.sql
│   └── src
│       ├── db.rs
│       ├── errors.rs
│       ├── lib.rs
│       ├── main.rs
│       ├── margin.rs
│       ├── manager.rs
│       ├── models.rs
│       ├── monitor.rs
│       ├── routes.rs
│       ├── service.rs
│       └── ws.rs
├── docs
│   ├── api.md
│   ├── architecture.md
│   ├── formulas.md
│   ├── risk_management.md
│   └── testing.md
└── programs
    └── position_manager
        ├── Cargo.toml
        └── src
            └── lib.rs
```

## Getting started

1. Install the required tooling:
   * Rust 1.75+
   * Anchor 0.29+
   * Solana CLI tools
   * PostgreSQL 14+
2. Build smart contract and backend:

```bash
anchor build
cargo build
```

3. Start the backend service (requires a running PostgreSQL instance and Solana RPC endpoint):

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost/positions
export SOLANA_CLUSTER=http://localhost:8899
export PROGRAM_ID=PosMgnmt111111111111111111111111111111111
cargo run -p position_backend
```

4. The REST API will be available at `http://localhost:8080` and WebSocket stream at `ws://localhost:8080/ws`.

## Documentation

Detailed design documentation is located in the `docs/` directory covering architecture, formulas, APIs, testing strategy, and risk management procedures.

## Disclaimer

Perpetual futures trading is inherently risky. The code in this repository is provided without warranty and should not be used to manage real funds without comprehensive security reviews.
