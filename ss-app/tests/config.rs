use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ss_app::config::{AppConfig, CliOverrides, EnvOverrides, StoreBackend};

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should advance")
        .as_nanos();
    std::env::temp_dir().join(format!("ss-app-{name}-{nanos}"))
}

#[test]
fn config_file_resolves_relative_paths_and_defaults() {
    let root = unique_temp_dir("config-file");
    fs::create_dir_all(root.join("frontend")).expect("create frontend dir");
    let config_path = root.join("ss-app.toml");

    fs::write(
        &config_path,
        r#"
[server]
listen = "127.0.0.1:9001"

[store]
backend = "fs"
root = "data-dir"

[frontend]
enabled = true
mount_path = "/app"
static_dir = "frontend"

[llm.apis.primary]
provider = "open_ai"
base_url = "http://localhost:11434/v1"
api_key = "demo-key"
model = "demo-model"
"#,
    )
    .expect("write config");

    let config = AppConfig::load_from_sources(
        CliOverrides {
            config: Some(config_path.clone()),
            ..CliOverrides::default()
        },
        EnvOverrides::default(),
    )
    .expect("load config");

    assert_eq!(config.server.listen, "127.0.0.1:9001");
    assert_eq!(config.store.backend, StoreBackend::Fs);
    assert_eq!(config.store.root, root.join("data-dir"));
    assert_eq!(config.frontend.mount_path, "/app");
    assert_eq!(config.frontend.static_dir, Some(root.join("frontend")));
    assert_eq!(config.llm.defaults.planner_api_id, "primary");
}

#[test]
fn env_overrides_file_values_and_creates_default_llm_api() {
    let root = unique_temp_dir("env-override");
    fs::create_dir_all(&root).expect("create root");
    let config_path = root.join("ss-app.toml");

    fs::write(
        &config_path,
        r#"
[server]
listen = "127.0.0.1:8080"

[store]
backend = "fs"
root = "data"
"#,
    )
    .expect("write config");

    let config = AppConfig::load_from_sources(
        CliOverrides {
            config: Some(config_path),
            ..CliOverrides::default()
        },
        EnvOverrides {
            listen: Some("127.0.0.1:9100".to_owned()),
            store_backend: Some(StoreBackend::Memory),
            default_openai_base_url: Some("http://localhost:3000/v1".to_owned()),
            default_openai_api_key: Some("env-key".to_owned()),
            default_openai_model: Some("env-model".to_owned()),
            ..EnvOverrides::default()
        },
    )
    .expect("load config");

    assert_eq!(config.server.listen, "127.0.0.1:9100");
    assert_eq!(config.store.backend, StoreBackend::Memory);
    assert_eq!(config.llm.defaults.director_api_id, "default");
    assert_eq!(
        config
            .llm
            .apis
            .get("default")
            .expect("default api should exist")
            .model,
        "env-model"
    );
}
