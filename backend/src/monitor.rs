use std::time::Duration;

use rust_decimal::Decimal;
use tokio::sync::broadcast;
use tokio::time::interval;
use tracing::warn;

use crate::errors::Result;
use crate::manager::PositionManager;
use crate::models::{PositionEvent, PositionEventType, PositionRecord};
use crate::ws::PositionBroadcast;

pub struct PositionMonitor {
    manager: PositionManager,
    broadcaster: PositionBroadcast,
    liquidation_threshold: Decimal,
}

impl PositionMonitor {
    pub fn new(manager: PositionManager, broadcaster: PositionBroadcast, liquidation_threshold: Decimal) -> Self {
        Self {
            manager,
            broadcaster,
            liquidation_threshold,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut ticker = interval(Duration::from_millis(1_000));
        loop {
            ticker.tick().await;
            self.scan_positions().await?;
        }
    }

    async fn scan_positions(&self) -> Result<()> {
        let mut alerts = Vec::new();
        for record in self.manager.cached_positions() {
            let mark_price = record.entry_price; // placeholder for oracle integration
            let collateral = record.margin + record.unrealized_pnl;
            let risk = self
                .manager
                .compute_risk_metrics(&record, mark_price, collateral)?;
            if risk.margin_ratio < self.liquidation_threshold {
                alerts.push(PositionEvent {
                    position_pubkey: record.position_pubkey.clone(),
                    owner: record.owner.clone(),
                    event_type: PositionEventType::LiquidationAlert,
                    payload: serde_json::json!({
                        "margin_ratio": risk.margin_ratio,
                        "liquidation_buffer": risk.liquidation_buffer,
                    }),
                    emitted_at: chrono::Utc::now(),
                });
            }
        }

        for alert in alerts {
            warn!("liquidation alert", position = %alert.position_pubkey, owner = %alert.owner);
            self.broadcaster.broadcast(alert.clone());
            self.manager.persist_event(alert).await?;
        }

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PositionEvent> {
        self.broadcaster.subscribe()
    }
}
