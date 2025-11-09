use thiserror::Error;

#[derive(Debug, Error)]
pub enum PositionServiceError {
    #[error("Anchor client error: {0}")]
    AnchorClient(String),
    #[error("Solana RPC error: {0}")]
    SolanaRpc(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Position not found: {0}")]
    PositionNotFound(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, PositionServiceError>;
