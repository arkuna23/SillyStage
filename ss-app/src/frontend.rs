use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use axum::extract::{OriginalUri, Request};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{MethodRouter, get};
use tower::util::ServiceExt;
use tower_http::services::ServeFile;

use crate::config::FrontendConfig;

pub fn mount_frontend(router: Router, config: &FrontendConfig) -> Router {
    if !config.enabled {
        return router;
    }

    match &config.static_dir {
        Some(static_dir) => mount_static_frontend(router, config, static_dir),
        None => mount_placeholder_frontend(router, config),
    }
}

fn mount_static_frontend(router: Router, config: &FrontendConfig, static_dir: &Path) -> Router {
    let mount_path = config.mount_path.as_str();
    let mount_slash = format!("{}/", mount_path.trim_end_matches('/'));
    let index_path = static_dir.join("index.html");

    if mount_path == "/" {
        if index_path.is_file() {
            let state =
                StaticFrontendState::new(config.mount_path.clone(), static_dir, &index_path);
            let handler = static_frontend_handler(state.clone());
            router
                .route("/", handler.clone())
                .route("/{*path}", handler)
        } else {
            router.route("/", placeholder_handler("/"))
        }
    } else if index_path.is_file() {
        let state = StaticFrontendState::new(config.mount_path.clone(), static_dir, &index_path);
        let handler = static_frontend_handler(state);
        let wildcard_route = format!("{mount_slash}{{*path}}");
        router
            .route(mount_path, handler.clone())
            .route(&mount_slash, handler.clone())
            .route(&wildcard_route, handler)
    } else {
        let redirect_path = mount_slash.clone();
        router
            .route(
                mount_path,
                get(move || {
                    let redirect_path = redirect_path.clone();
                    async move { Redirect::temporary(&redirect_path) }
                }),
            )
            .route(&mount_slash, placeholder_handler(mount_path))
    }
}

fn mount_placeholder_frontend(router: Router, config: &FrontendConfig) -> Router {
    let mount_path = config.mount_path.clone();
    let mount_slash = format!("{}/", mount_path.trim_end_matches('/'));
    let handler = placeholder_handler(&mount_path);

    if mount_path == "/" {
        router.route("/", handler)
    } else {
        router
            .route(&mount_path, handler.clone())
            .route(&mount_slash, handler)
    }
}

fn static_frontend_handler(state: StaticFrontendState) -> MethodRouter {
    get(move |uri: OriginalUri, request: Request| {
        let state = state.clone();
        async move { serve_static_frontend(state, uri, request).await }
    })
}

fn placeholder_handler(mount_path: &str) -> MethodRouter {
    let html = placeholder_html(mount_path);
    get(move || {
        let html = html.clone();
        async move { Html(html) }
    })
}

fn placeholder_html(mount_path: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>SillyStage</title>
    <style>
      body {{
        margin: 0;
        font-family: ui-sans-serif, system-ui, sans-serif;
        background: linear-gradient(135deg, #16181d, #252a34);
        color: #f4f6fb;
      }}
      main {{
        max-width: 760px;
        margin: 10vh auto;
        padding: 32px;
      }}
      h1 {{ margin: 0 0 12px; font-size: 40px; }}
      p {{ line-height: 1.6; color: #d6dbe8; }}
      code {{
        background: rgba(255,255,255,0.08);
        padding: 2px 6px;
        border-radius: 6px;
      }}
    </style>
  </head>
  <body>
    <main>
      <h1>SillyStage</h1>
      <p>The backend is running. The web frontend is not implemented yet.</p>
      <p>Protocol entry: <code>POST /rpc</code></p>
      <p>Health check: <code>GET /healthz</code></p>
      <p>Frontend mount path: <code>{mount_path}</code></p>
    </main>
  </body>
</html>"#
    )
}

#[derive(Clone)]
struct StaticFrontendState {
    mount_path: Arc<String>,
    static_dir: Arc<PathBuf>,
    index_path: Arc<PathBuf>,
}

impl StaticFrontendState {
    fn new(mount_path: String, static_dir: &Path, index_path: &Path) -> Self {
        Self {
            mount_path: Arc::new(mount_path),
            static_dir: Arc::new(static_dir.to_path_buf()),
            index_path: Arc::new(index_path.to_path_buf()),
        }
    }
}

async fn serve_static_frontend(
    state: StaticFrontendState,
    uri: OriginalUri,
    request: Request,
) -> Response {
    let Some(relative_path) = frontend_relative_path(&state.mount_path, uri.path()) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let candidate = if relative_path.is_empty() {
        state.index_path.as_ref().clone()
    } else {
        state.static_dir.join(&relative_path)
    };

    if candidate.is_file() {
        return serve_file_response(candidate, request).await;
    }

    if relative_path.is_empty() || !path_looks_like_static_asset(&relative_path) {
        return serve_file_response(state.index_path.as_ref().clone(), request).await;
    }

    StatusCode::NOT_FOUND.into_response()
}

async fn serve_file_response(path: PathBuf, request: Request) -> Response {
    match ServeFile::new(path).oneshot(request).await {
        Ok(response) => response.into_response(),
        Err(_error) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn frontend_relative_path(mount_path: &str, request_path: &str) -> Option<String> {
    if mount_path == "/" {
        return Some(request_path.trim_start_matches('/').to_owned());
    }

    let prefix = mount_path.trim_end_matches('/');
    let relative = request_path.strip_prefix(prefix)?;
    Some(relative.trim_start_matches('/').to_owned())
}

fn path_looks_like_static_asset(path: &str) -> bool {
    let last_segment = path.rsplit('/').next().unwrap_or(path);
    !last_segment.is_empty() && last_segment.contains('.')
}
