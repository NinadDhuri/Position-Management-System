use std::sync::Arc;

use anchor_client::{solana_sdk::signature::Keypair, Client, Cluster, Program};
use dashmap::DashMap;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use tracing::info;

use crate::db::Database;
use crate::errors::{PositionServiceError, Result};
use crate::margin::{calculate_liquidation_price_long, calculate_liquidation_price_short, calculate_margin_ratio, calculate_unrealized_pnl};
use crate::models::{ClosePositionRequest, MarginRequirement, OpenPositionRequest, PositionEvent, PositionEventType, PositionRecord, PositionSide, PositionState, RiskMetrics};

#[derive(Clone)]
pub struct PositionManager {
    pub program: Program,
    pub db: Database,
    cache: Arc<DashMap<String, PositionRecord>>,
}

impl PositionManager {
    pub fn new(cluster: Cluster, payer: Keypair, program_id: Pubkey, db: Database) -> Result<Self> {
        let client = Client::new(cluster, payer);
        let program = client.program(program_id);
        Ok(Self {
            program,
            db,
            cache: Arc::new(DashMap::new()),
        })
    }

    pub async fn cache_position(&self, record: PositionRecord) {
        self.cache.insert(record.position_pubkey.clone(), record);
    }

    pub fn cached_positions(&self) -> Vec<PositionRecord> {
        self.cache.iter().map(|entry| entry.value().clone()).collect()
    }

    pub fn get_cached_position(&self, position_pubkey: &str) -> Option<PositionRecord> {
        self.cache.get(position_pubkey).map(|entry| entry.value().clone())
    }

    pub async fn open_position(&self, owner: &Pubkey, request: OpenPositionRequest) -> Result<()> {
        if request.size <= Decimal::ZERO {
            return Err(PositionServiceError::Validation("position size must be positive".into()));
        }
        if request.entry_price <= Decimal::ZERO {
            return Err(PositionServiceError::Validation("entry price must be positive".into()));
        }
        if request.collateral <= Decimal::ZERO {
            return Err(PositionServiceError::Validation("collateral must be positive".into()));
        }
        let margin_requirement = self.compute_margin_requirement(&request)?;
        let liquidation_price = match request.side {
            PositionSide::Long => calculate_liquidation_price_long(
                request.entry_price,
                Decimal::from(request.leverage),
                margin_requirement.maintenance_margin / request.size,
            )?,
            PositionSide::Short => calculate_liquidation_price_short(
                request.entry_price,
                Decimal::from(request.leverage),
                margin_requirement.maintenance_margin / request.size,
            )?,
        };

        if request.collateral < margin_requirement.initial_margin {
            return Err(PositionServiceError::Validation("insufficient collateral".into()));
        }

        let margin_ratio = calculate_margin_ratio(
            request.collateral,
            Decimal::ZERO,
            request.size,
            request.entry_price,
        )?;

        let record = PositionRecord {
            position_pubkey: "pending".into(),
            owner: owner.to_string(),
            symbol: request.symbol.clone(),
            side: request.side.clone(),
            size: request.size,
            entry_price: request.entry_price,
            margin: margin_requirement.initial_margin,
            leverage: request.leverage,
            unrealized_pnl: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
            funding_accrued: Decimal::ZERO,
            liquidation_price,
            maintenance_margin: margin_requirement.maintenance_margin,
            margin_ratio,
            state: PositionState::Opening,
            last_update: chrono::Utc::now(),
        };

        self.cache_position(record).await;

        info!("queued open position", owner = %owner, symbol = %request.symbol);
        Ok(())
    }

    pub async fn close_position(&self, position_pubkey: &Pubkey, request: ClosePositionRequest) -> Result<()> {
        let mut record = self
            .get_cached_position(&position_pubkey.to_string())
            .ok_or_else(|| PositionServiceError::PositionNotFound(position_pubkey.to_string()))?;
        let pnl = calculate_unrealized_pnl(
            record.side == PositionSide::Long,
            record.size,
            request.exit_price,
            record.entry_price,
        );
        record.realized_pnl += pnl - request.funding_payment;
        record.unrealized_pnl = Decimal::ZERO;
        record.state = PositionState::Closed;
        record.last_update = chrono::Utc::now();
        self.cache_position(record.clone()).await;

        self.persist_event(PositionEvent {
            position_pubkey: position_pubkey.to_string(),
            owner: record.owner.clone(),
            event_type: PositionEventType::Closed,
            payload: serde_json::json!({
                "exit_price": request.exit_price,
                "funding_payment": request.funding_payment,
                "pnl": record.realized_pnl,
            }),
            emitted_at: chrono::Utc::now(),
        })
        .await?;

        Ok(())
    }

    pub fn compute_margin_requirement(&self, request: &OpenPositionRequest) -> Result<MarginRequirement> {
        let notional = request.size * request.entry_price;
        let initial_margin = notional / Decimal::from(request.leverage);
        let maintenance_margin = initial_margin * Decimal::from_f64(0.5).unwrap();
        Ok(MarginRequirement {
            initial_margin,
            maintenance_margin,
            leverage: request.leverage,
        })
    }

    pub fn compute_risk_metrics(&self, record: &PositionRecord, mark_price: Decimal, collateral: Decimal) -> Result<RiskMetrics> {
        let unrealized = calculate_unrealized_pnl(record.side == PositionSide::Long, record.size, mark_price, record.entry_price);
        let margin_ratio = calculate_margin_ratio(collateral, unrealized, record.size, mark_price)?;
        let liquidation_buffer = (mark_price - record.liquidation_price).abs();
        Ok(RiskMetrics {
            margin_ratio,
            liquidation_buffer,
            maintenance_margin: record.maintenance_margin,
            initial_margin: record.margin,
        })
    }

    pub async fn persist_event(&self, event: PositionEvent) -> Result<()> {
        let client = self.db.client().await?;
        client
            .execute(
                "INSERT INTO position_events (position_pubkey, owner_pubkey, event_type, payload, emitted_at) VALUES ($1, $2, $3, $4, $5)",
                &[&event.position_pubkey, &event.owner, &event.event_type.as_ref(), &event.payload, &event.emitted_at],
            )
            .await
            .map_err(|e| PositionServiceError::Database(e.to_string()))?;
        Ok(())
    }
}
