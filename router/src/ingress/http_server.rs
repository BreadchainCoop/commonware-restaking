use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use std::sync::Arc;
use tracing::info;

use crate::ingress::types::{TaskRequest, TaskResponse};
use crate::usecases::counter::creator::{SimpleTaskQueue, TaskQueue};

// Handler for POST /trigger
pub async fn trigger_task_handler(
    State(queue): State<Arc<SimpleTaskQueue>>,
    Json(req): Json<TaskRequest>,
) -> (StatusCode, Json<TaskResponse>) {
    // Add business logic here such as api-key verification, ecdsa signature verification, etc retrieved from the TaskRequest
    // for example, if we assume `var1` is the api-key
    // let var1 = req.body.var1;
    // if !is_valid_api_key(var1) {
    //     return (StatusCode::UNAUTHORIZED, Json(TaskResponse {
    //         success: false,
    //         message: "Invalid api-key".to_string(),
    //     }));
    // }

    // Add to queue using the TaskQueue trait method
    queue.push(req);
    (
        StatusCode::OK,
        Json(TaskResponse {
            success: true,
            message: "Task queued".to_string(),
        }),
    )
}

// Start the HTTP server in a background task
pub async fn start_http_server(queue: Arc<SimpleTaskQueue>, addr: &str) {
    let app = Router::new()
        .route("/trigger", post(trigger_task_handler))
        .with_state(queue);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind HTTP server");
    info!("Creator HTTP server running on {}", addr);
    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}
