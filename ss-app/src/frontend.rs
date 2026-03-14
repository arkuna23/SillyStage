use std::path::Path;

use axum::Router;
use axum::response::{Html, Redirect};
use axum::routing::{MethodRouter, get, get_service};
use tower_http::services::{ServeDir, ServeFile};

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
    let static_service = get_service(ServeDir::new(static_dir));

    if mount_path == "/" {
        if index_path.is_file() {
            let index_service = get_service(ServeFile::new(index_path));
            router
                .route_service("/", index_service)
                .fallback_service(static_service)
        } else {
            router
                .route("/", placeholder_handler("/"))
                .fallback_service(static_service)
        }
    } else if index_path.is_file() {
        let index_service = get_service(ServeFile::new(index_path));
        router
            .route_service(mount_path, index_service.clone())
            .route_service(&mount_slash, index_service)
            .nest_service(mount_path, static_service)
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
            .nest_service(mount_path, static_service)
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
