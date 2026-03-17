use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use handler::Handler;

use crate::ServerState;

use super::binary::{download, upload};
use super::rpc::handle_rpc;

pub fn build_router(handler: Arc<Handler>) -> Router {
    Router::new()
        .route("/upload/{resource_id}/{file_id}", post(upload))
        .route("/download/{resource_id}/{file_id}", get(download))
        .route("/rpc", post(handle_rpc))
        .route("/healthz", get(healthz))
        .with_state(ServerState::new(handler))
}

async fn healthz() -> &'static str {
    "ok"
}
