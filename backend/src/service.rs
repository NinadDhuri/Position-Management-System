use std::net::SocketAddr;

use axum::Router;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use tokio::task::JoinHandle;
use tracing::info;

use crate::manager::PositionManager;
use crate::monitor::PositionMonitor;
use crate::routes::{api_router, AppState};
use crate::ws::PositionBroadcast;

pub struct PositionService {
    router: Router,
    _monitor_handle: JoinHandle<()>,
}

impl PositionService {
    pub fn new(manager: PositionManager) -> Self {
        let broadcaster = PositionBroadcast::new(1024);
        let monitor = PositionMonitor::new(manager.clone(), broadcaster.clone(), Decimal::from_f64(0.05).unwrap());
        let monitor_handle = tokio::spawn(async move {
            if let Err(err) = monitor.start().await {
                tracing::error!("monitor error: {err}");
            }
        });
        let state = AppState { manager, broadcaster };
        let router = api_router(state);
        Self { router, _monitor_handle: monitor_handle }
    }

    pub async fn run(self, addr: SocketAddr) {
        info!("starting position management service", %addr);
        if let Err(err) = axum::Server::bind(&addr).serve(self.router.into_make_service()).await {
            tracing::error!("server error: {err}");
        }
    }
}
