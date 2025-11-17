use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use std::sync::Arc;
use tracing::info;

use crate::creator::{SimpleTaskQueue, TaskQueue, TaskRequest, TaskResponse};

pub async fn trigger_task_handler(
    State(queue): State<Arc<SimpleTaskQueue>>,
    Json(req): Json<TaskRequest>,
) -> (StatusCode, Json<TaskResponse>) {
    queue.push(req);
    (
        StatusCode::OK,
        Json(TaskResponse {
            success: true,
            message: "Task queued".to_string(),
        }),
    )
}

pub async fn start_http_server(queue: Arc<SimpleTaskQueue>, addr: &str) {
    let app = Router::new()
        .route("/trigger", post(trigger_task_handler))
        .with_state(queue);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind HTTP server");
    info!("Counter Creator HTTP server running on {}", addr);
    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}
