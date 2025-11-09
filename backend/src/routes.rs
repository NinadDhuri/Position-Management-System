use std::str::FromStr;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

use crate::errors::PositionServiceError;
use crate::manager::PositionManager;
use crate::models::{ClosePositionRequest, ModifyPositionRequest, OpenPositionRequest, PositionRecord, UserPositionsResponse};
use crate::ws::{position_ws_handler, PositionBroadcast};

#[derive(Clone)]
pub struct AppState {
    pub manager: PositionManager,
    pub broadcaster: PositionBroadcast,
}

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/positions/open", post(open_position))
        .route("/positions/:id/modify", put(modify_position))
        .route("/positions/:id/close", delete(close_position))
        .route("/positions/:id", get(get_position))
        .route("/users/:id/positions", get(get_user_positions))
        .route("/ws", get(position_ws_handler))
        .with_state(state)
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid public key: {0}")]
    InvalidPubkey(String),
    #[error(transparent)]
    Service(#[from] PositionServiceError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::InvalidPubkey(_) => StatusCode::BAD_REQUEST,
            ApiError::Service(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

async fn open_position(State(state): State<AppState>, Json(payload): Json<OpenPositionRequest>) -> Result<Json<PositionRecord>, ApiError> {
    let owner = Pubkey::new_unique();
    state.manager.open_position(&owner, payload.clone()).await?;
    let record = state
        .manager
        .cached_positions()
        .into_iter()
        .find(|pos| pos.symbol == payload.symbol)
        .ok_or_else(|| PositionServiceError::Internal("Position not cached".into()))?;
    Ok(Json(record))
}

async fn modify_position(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(_payload): Json<ModifyPositionRequest>,
) -> Result<Json<PositionRecord>, ApiError> {
    let record = state
        .manager
        .get_cached_position(&id)
        .ok_or_else(|| PositionServiceError::PositionNotFound(id.clone()))?;
    Ok(Json(record))
}

async fn close_position(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ClosePositionRequest>,
) -> Result<StatusCode, ApiError> {
    let pubkey = Pubkey::from_str(&id).map_err(|_| ApiError::InvalidPubkey(id.clone()))?;
    state.manager.close_position(&pubkey, payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_position(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<PositionRecord>, ApiError> {
    let record = state
        .manager
        .get_cached_position(&id)
        .ok_or_else(|| PositionServiceError::PositionNotFound(id.clone()))?;
    Ok(Json(record))
}

async fn get_user_positions(State(state): State<AppState>, Path(owner): Path<String>) -> Result<Json<UserPositionsResponse>, ApiError> {
    let positions: Vec<PositionRecord> = state
        .manager
        .cached_positions()
        .into_iter()
        .filter(|record| record.owner == owner)
        .collect();
    Ok(Json(UserPositionsResponse { owner, positions }))
}

