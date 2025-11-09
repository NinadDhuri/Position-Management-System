use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum PositionState {
    Opening,
    Open,
    Modifying,
    Closing,
    Closed,
    Liquidating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRecord {
    pub position_pubkey: String,
    pub owner: String,
    pub symbol: String,
    pub side: PositionSide,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub margin: Decimal,
    pub leverage: u16,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub funding_accrued: Decimal,
    pub liquidation_price: Decimal,
    pub maintenance_margin: Decimal,
    pub margin_ratio: Decimal,
    pub state: PositionState,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSnapshot {
    pub position_pubkey: String,
    pub timestamp: DateTime<Utc>,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub mark_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub margin_ratio: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionEvent {
    pub position_pubkey: String,
    pub owner: String,
    pub event_type: PositionEventType,
    pub payload: serde_json::Value,
    pub emitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, AsRefStr, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum PositionEventType {
    Opened,
    Modified,
    Closed,
    LiquidationAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginRequirement {
    pub initial_margin: Decimal,
    pub maintenance_margin: Decimal,
    pub leverage: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyPositionRequest {
    pub size_delta: Decimal,
    pub margin_delta: Decimal,
    pub new_leverage: Option<u16>,
    pub new_entry_price: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPositionRequest {
    pub symbol: String,
    pub side: PositionSide,
    pub size: Decimal,
    pub leverage: u16,
    pub entry_price: Decimal,
    pub collateral: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosePositionRequest {
    pub exit_price: Decimal,
    pub funding_payment: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPositionsResponse {
    pub owner: String,
    pub positions: Vec<PositionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub margin_ratio: Decimal,
    pub liquidation_buffer: Decimal,
    pub maintenance_margin: Decimal,
    pub initial_margin: Decimal,
}

