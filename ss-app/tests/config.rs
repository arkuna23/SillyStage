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
open_browser = false

[store]
backend = "fs"
root = "data-dir"

[frontend]
enabled = true
mount_path = "/app"
static_dir = "frontend"
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
    assert!(!config.server.open_browser);
    assert_eq!(config.store.backend, StoreBackend::Fs);
    assert_eq!(config.store.root, root.join("data-dir"));
    assert_eq!(config.frontend.mount_path, "/app");
    assert_eq!(config.frontend.static_dir, Some(root.join("frontend")));
}

#[test]
fn env_overrides_file_values() {
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
            open_browser: Some(false),
            store_backend: Some(StoreBackend::Memory),
            ..EnvOverrides::default()
        },
    )
    .expect("load config");

    assert_eq!(config.server.listen, "127.0.0.1:9100");
    assert!(!config.server.open_browser);
    assert_eq!(config.store.backend, StoreBackend::Memory);
}
