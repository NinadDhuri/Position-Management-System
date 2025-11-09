use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::{Config, NoTls};

use crate::errors::{PositionServiceError, Result};

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub fn new(database_url: &str) -> Result<Self> {
        let pg_config: Config = database_url
            .parse()
            .map_err(|e| PositionServiceError::Database(e.to_string()))?;

        let mgr = Manager::from_config(pg_config, NoTls, ManagerConfig { recycling_method: RecyclingMethod::Fast });
        let pool = Pool::builder(mgr)
            .max_size(16)
            .build()
            .map_err(|e| PositionServiceError::Database(e.to_string()))?;
        Ok(Self { pool })
    }

    pub async fn client(&self) -> Result<deadpool_postgres::Client> {
        self.pool
            .get()
            .await
            .map_err(|e| PositionServiceError::Database(e.to_string()))
    }

    pub async fn migrate(&self) -> Result<()> {
        let client = self.client().await?;
        let tx = client
            .build_transaction()
            .start()
            .await
            .map_err(|e| PositionServiceError::Database(e.to_string()))?;

        tx.batch_execute(include_str!("../sql/schema.sql"))
            .await
            .map_err(|e| PositionServiceError::Database(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| PositionServiceError::Database(e.to_string()))
    }
}
