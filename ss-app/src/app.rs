use std::sync::Arc;

use axum::Router;
use handler::Handler;
use store::{FileSystemStore, InMemoryStore, Store};
use tokio::net::TcpListener;
use tracing::info;

use crate::browser::spawn_browser_if_desktop;
use crate::config::{AppConfig, FrontendConfig, StoreBackend};
use crate::error::AppError;
use crate::frontend::mount_frontend;
use crate::llm::seed_store_and_build_registry;

pub async fn build_store(config: &AppConfig) -> Result<Arc<dyn Store>, AppError> {
    match config.store.backend {
        StoreBackend::Fs => Ok(Arc::new(FileSystemStore::new(&config.store.root).await?)),
        StoreBackend::Memory => Ok(Arc::new(InMemoryStore::new())),
    }
}

pub async fn build_handler(config: &AppConfig) -> Result<Arc<Handler>, AppError> {
    let store = build_store(config).await?;
    let (registry, effective_default_llm_config) =
        seed_store_and_build_registry(&store, config).await?;
    let handler = Handler::new(
        store,
        registry,
        config.llm.defaults.clone(),
        effective_default_llm_config,
    )
    .await?;
    Ok(Arc::new(handler))
}

pub async fn build_router(config: &AppConfig) -> Result<Router, AppError> {
    let handler = build_handler(config).await?;
    Ok(build_router_with_handler(handler, &config.frontend))
}

pub fn build_router_with_handler(handler: Arc<Handler>, frontend: &FrontendConfig) -> Router {
    let router = server::http::build_router(handler);
    mount_frontend(router, frontend)
}

pub async fn run(config: AppConfig) -> Result<(), AppError> {
    let router = build_router(&config).await?;
    let listener = TcpListener::bind(&config.server.listen).await?;
    let local_addr = listener.local_addr()?;
    info!(listen = %local_addr, "SillyStage listening");
    spawn_browser_if_desktop(&config, local_addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
