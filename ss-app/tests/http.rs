use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use protocol::{
    ConfigGetGlobalParams, JsonRpcRequestMessage, JsonRpcResponseMessage, RequestParams,
};
use ss_app::app::build_router;
use ss_app::config::{
    AppConfig, FrontendConfig, LlmApiConfig, LlmConfig, LlmProvider, ServerConfig, StoreBackend,
    StoreConfig,
};
use store::AgentApiIds;
use tower::util::ServiceExt;

fn app_config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            listen: "127.0.0.1:0".to_owned(),
            open_browser: false,
        },
        store: StoreConfig {
            backend: StoreBackend::Memory,
            root: std::path::PathBuf::from("./unused"),
        },
        frontend: FrontendConfig {
            enabled: true,
            mount_path: "/".to_owned(),
            static_dir: None,
        },
        llm: LlmConfig {
            apis: [(
                "default".to_owned(),
                LlmApiConfig {
                    provider: LlmProvider::OpenAi,
                    base_url: "http://localhost:11434/v1".to_owned(),
                    api_key: "demo-key".to_owned(),
                    model: "demo-model".to_owned(),
                    temperature: None,
                    max_tokens: None,
                },
            )]
            .into_iter()
            .collect(),
            defaults: Some(AgentApiIds {
                planner_api_id: "default".to_owned(),
                architect_api_id: "default".to_owned(),
                director_api_id: "default".to_owned(),
                actor_api_id: "default".to_owned(),
                narrator_api_id: "default".to_owned(),
                keeper_api_id: "default".to_owned(),
                replyer_api_id: "default".to_owned(),
            }),
            default_config: None,
        },
    }
}

struct TempFrontendDir {
    path: std::path::PathBuf,
}

impl TempFrontendDir {
    fn new() -> Self {
        let unique = format!(
            "sillystage-ss-app-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(path.join("assets")).expect("create temp frontend dir");
        std::fs::write(
            path.join("index.html"),
            "<!doctype html><html><body>spa-entry</body></html>",
        )
        .expect("write index");
        std::fs::write(path.join("assets/app.js"), "console.log('ok');").expect("write asset");
        Self { path }
    }
}

impl Drop for TempFrontendDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn static_frontend_config(mount_path: &str) -> (AppConfig, TempFrontendDir) {
    let mut config = app_config();
    let temp_dir = TempFrontendDir::new();
    config.frontend.static_dir = Some(temp_dir.path.clone());
    config.frontend.mount_path = mount_path.to_owned();
    (config, temp_dir)
}

#[tokio::test]
async fn root_returns_placeholder_page() {
    let router = build_router(&app_config()).await.expect("build router");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let text = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(text.contains("SillyStage"));
    assert!(text.contains("POST /rpc"));
}

#[tokio::test]
async fn healthz_and_rpc_routes_are_wired() {
    let router = build_router(&app_config()).await.expect("build router");

    let healthz = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(healthz.status(), StatusCode::OK);

    let rpc_request = JsonRpcRequestMessage::new(
        "req-1",
        None::<String>,
        RequestParams::ConfigGetGlobal(ConfigGetGlobalParams {}),
    );
    let rpc_response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&rpc_request).expect("serialize request"),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(rpc_response.status(), StatusCode::OK);
    let body = to_bytes(rpc_response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let _: JsonRpcResponseMessage = serde_json::from_slice(&body).expect("valid json-rpc response");
}

#[tokio::test]
async fn static_frontend_root_mount_serves_spa_routes_without_breaking_backend_routes() {
    let (config, _temp_dir) = static_frontend_config("/");
    let router = build_router(&config).await.expect("build router");

    let spa = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/workspace/dashboard")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(spa.status(), StatusCode::OK);
    let spa_body = to_bytes(spa.into_body(), usize::MAX)
        .await
        .expect("read body");
    let spa_text = String::from_utf8(spa_body.to_vec()).expect("utf8");
    assert!(spa_text.contains("spa-entry"));

    let missing_asset = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/assets/missing.js")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(missing_asset.status(), StatusCode::NOT_FOUND);

    let healthz = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/healthz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(healthz.status(), StatusCode::OK);

    let rpc_request = JsonRpcRequestMessage::new(
        "req-static-root",
        None::<String>,
        RequestParams::ConfigGetGlobal(ConfigGetGlobalParams {}),
    );
    let rpc_response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&rpc_request).expect("serialize request"),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(rpc_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn static_frontend_nested_mount_serves_spa_routes() {
    let (config, _temp_dir) = static_frontend_config("/app");
    let router = build_router(&config).await.expect("build router");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/workspace/dashboard")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read body");
    let text = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(text.contains("spa-entry"));
}
