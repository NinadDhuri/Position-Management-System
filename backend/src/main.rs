use std::str::FromStr;
use std::net::SocketAddr;

use anchor_client::Cluster;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use tracing_subscriber::EnvFilter;

use position_backend::db::Database;
use position_backend::manager::PositionManager;
use position_backend::service::PositionService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost/positions".into());
    let program_id = std::env::var("PROGRAM_ID")
        .ok()
        .and_then(|v| v.parse::<Pubkey>().ok())
        .unwrap_or_else(|| Pubkey::from_str("PosMgnmt111111111111111111111111111111111").unwrap());
    let rpc_url = std::env::var("SOLANA_CLUSTER").unwrap_or_else(|_| "http://localhost:8899".into());
    let payer = Keypair::new();

    let db = Database::new(&database_url)?;
    db.migrate().await?;

    let manager = PositionManager::new(Cluster::Custom(rpc_url.clone(), rpc_url), payer, program_id, db.clone())?;
    let service = PositionService::new(manager);
    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    service.run(addr).await;
    Ok(())
}
