use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::StreamExt;
use tokio::select;
use tokio::sync::broadcast;

use crate::models::PositionEvent;
use crate::routes::AppState;

#[derive(Clone)]
pub struct PositionBroadcast {
    sender: broadcast::Sender<PositionEvent>,
}

impl PositionBroadcast {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn broadcast(&self, event: PositionEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PositionEvent> {
        self.sender.subscribe()
    }
}

pub async fn position_ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        handle_ws(socket, state.broadcaster.clone()).await;
    })
}

async fn handle_ws(mut socket: WebSocket, broadcaster: PositionBroadcast) {
    let mut receiver = broadcaster.subscribe();
    loop {
        select! {
            message = socket.next() => {
                if message.is_none() {
                    break;
                }
            }
            event = receiver.recv() => {
                match event {
                    Ok(event) => {
                        if socket.send(Message::Text(serde_json::to_string(&event).unwrap())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
