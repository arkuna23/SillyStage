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
                },
            )]
            .into_iter()
            .collect(),
            defaults: AgentApiIds {
                planner_api_id: "default".to_owned(),
                architect_api_id: "default".to_owned(),
                director_api_id: "default".to_owned(),
                actor_api_id: "default".to_owned(),
                narrator_api_id: "default".to_owned(),
                keeper_api_id: "default".to_owned(),
            },
        },
    }
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
