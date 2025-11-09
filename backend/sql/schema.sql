CREATE TABLE IF NOT EXISTS users (
    owner_pubkey TEXT PRIMARY KEY,
    total_collateral NUMERIC(32, 8) NOT NULL DEFAULT 0,
    locked_collateral NUMERIC(32, 8) NOT NULL DEFAULT 0,
    total_pnl NUMERIC(32, 8) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS positions (
    position_pubkey TEXT PRIMARY KEY,
    owner_pubkey TEXT NOT NULL REFERENCES users(owner_pubkey),
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    size NUMERIC(32, 8) NOT NULL,
    entry_price NUMERIC(32, 8) NOT NULL,
    margin NUMERIC(32, 8) NOT NULL,
    leverage INT NOT NULL,
    unrealized_pnl NUMERIC(32, 8) NOT NULL DEFAULT 0,
    realized_pnl NUMERIC(32, 8) NOT NULL DEFAULT 0,
    funding_accrued NUMERIC(32, 8) NOT NULL DEFAULT 0,
    liquidation_price NUMERIC(32, 8) NOT NULL,
    maintenance_margin NUMERIC(32, 8) NOT NULL,
    margin_ratio NUMERIC(32, 8) NOT NULL DEFAULT 0,
    state TEXT NOT NULL,
    last_update TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS position_events (
    id BIGSERIAL PRIMARY KEY,
    position_pubkey TEXT NOT NULL REFERENCES positions(position_pubkey),
    owner_pubkey TEXT NOT NULL REFERENCES users(owner_pubkey),
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    emitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS position_snapshots (
    id BIGSERIAL PRIMARY KEY,
    position_pubkey TEXT NOT NULL REFERENCES positions(position_pubkey),
    snapshot_time TIMESTAMPTZ NOT NULL,
    size NUMERIC(32, 8) NOT NULL,
    entry_price NUMERIC(32, 8) NOT NULL,
    mark_price NUMERIC(32, 8) NOT NULL,
    unrealized_pnl NUMERIC(32, 8) NOT NULL,
    margin_ratio NUMERIC(32, 8) NOT NULL
);

CREATE TABLE IF NOT EXISTS risk_metrics (
    id BIGSERIAL PRIMARY KEY,
    owner_pubkey TEXT NOT NULL REFERENCES users(owner_pubkey),
    symbol TEXT NOT NULL,
    margin_ratio NUMERIC(32, 8) NOT NULL,
    maintenance_margin NUMERIC(32, 8) NOT NULL,
    initial_margin NUMERIC(32, 8) NOT NULL,
    liquidation_buffer NUMERIC(32, 8) NOT NULL,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_positions_owner ON positions(owner_pubkey);
CREATE INDEX IF NOT EXISTS idx_positions_symbol ON positions(symbol);
CREATE INDEX IF NOT EXISTS idx_position_events_position ON position_events(position_pubkey);
CREATE INDEX IF NOT EXISTS idx_position_snapshots_position ON position_snapshots(position_pubkey);
