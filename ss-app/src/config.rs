use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub store: StoreConfig,
    pub frontend: FrontendConfig,
}

impl AppConfig {
    pub fn load(cli: CliOverrides) -> Result<Self, ConfigError> {
        Self::load_from_sources(cli, EnvOverrides::from_env()?)
    }

    pub fn load_from_sources(cli: CliOverrides, env: EnvOverrides) -> Result<Self, ConfigError> {
        let config_path = resolve_config_path(&cli, &env);
        let mut resolved = ResolvedConfig::default();

        if let Some(path) = config_path {
            let content =
                fs::read_to_string(&path).map_err(|source| ConfigError::ReadConfigFile {
                    path: path.clone(),
                    source,
                })?;
            let file_config = toml::from_str::<FileConfig>(&content).map_err(|source| {
                ConfigError::ParseConfigFile {
                    path: path.clone(),
                    source,
                }
            })?;
            let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
            resolved.apply_file(file_config, base_dir);
        }

        resolved.apply_env(env)?;
        resolved.apply_cli(cli);
        resolved.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub listen: String,
    pub open_browser: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum StoreBackend {
    Fs,
    Memory,
}

impl StoreBackend {
    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "fs" => Ok(Self::Fs),
            "memory" => Ok(Self::Memory),
            other => Err(ConfigError::InvalidStoreBackend(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreConfig {
    pub backend: StoreBackend,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendConfig {
    pub enabled: bool,
    pub mount_path: String,
    pub static_dir: Option<PathBuf>,
}

#[derive(Parser, Debug, Clone, Default)]
#[command(name = "ss-app")]
pub struct Cli {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub listen: Option<String>,
    #[arg(long)]
    pub store_root: Option<PathBuf>,
    #[arg(long, value_enum)]
    pub store_backend: Option<StoreBackend>,
}

#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub config: Option<PathBuf>,
    pub listen: Option<String>,
    pub store_root: Option<PathBuf>,
    pub store_backend: Option<StoreBackend>,
}

impl From<Cli> for CliOverrides {
    fn from(value: Cli) -> Self {
        Self {
            config: value.config,
            listen: value.listen,
            store_root: value.store_root,
            store_backend: value.store_backend,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EnvOverrides {
    pub config: Option<PathBuf>,
    pub listen: Option<String>,
    pub open_browser: Option<bool>,
    pub store_root: Option<PathBuf>,
    pub store_backend: Option<StoreBackend>,
    pub frontend_enabled: Option<bool>,
    pub frontend_mount_path: Option<String>,
    pub frontend_static_dir: Option<PathBuf>,
}

impl EnvOverrides {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            config: env::var_os("SS_APP_CONFIG").map(PathBuf::from),
            listen: env::var("SS_APP_LISTEN").ok(),
            open_browser: env::var("SS_APP_OPEN_BROWSER")
                .ok()
                .map(|value| parse_bool_env("SS_APP_OPEN_BROWSER", &value))
                .transpose()?,
            store_root: env::var_os("SS_APP_STORE_ROOT").map(PathBuf::from),
            store_backend: env::var("SS_APP_STORE_BACKEND")
                .ok()
                .map(|value| StoreBackend::parse(&value))
                .transpose()?,
            frontend_enabled: env::var("SS_APP_FRONTEND_ENABLED")
                .ok()
                .map(|value| parse_bool_env("SS_APP_FRONTEND_ENABLED", &value))
                .transpose()?,
            frontend_mount_path: env::var("SS_APP_FRONTEND_MOUNT_PATH").ok(),
            frontend_static_dir: env::var_os("SS_APP_FRONTEND_STATIC_DIR").map(PathBuf::from),
        })
    }
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    server: Option<FileServerConfig>,
    store: Option<FileStoreConfig>,
    frontend: Option<FileFrontendConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct FileServerConfig {
    listen: Option<String>,
    open_browser: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct FileStoreConfig {
    backend: Option<StoreBackend>,
    root: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct FileFrontendConfig {
    enabled: Option<bool>,
    mount_path: Option<String>,
    static_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct ResolvedConfig {
    server: ServerConfig,
    store: StoreConfig,
    frontend: FrontendConfig,
}

impl Default for ResolvedConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                listen: "127.0.0.1:8080".to_owned(),
                open_browser: true,
            },
            store: StoreConfig {
                backend: StoreBackend::Fs,
                root: PathBuf::from("./data"),
            },
            frontend: FrontendConfig {
                enabled: true,
                mount_path: "/".to_owned(),
                static_dir: None,
            },
        }
    }
}

impl ResolvedConfig {
    fn apply_file(&mut self, file: FileConfig, base_dir: &Path) {
        if let Some(server) = file.server {
            if let Some(listen) = server.listen {
                self.server.listen = listen;
            }
            if let Some(open_browser) = server.open_browser {
                self.server.open_browser = open_browser;
            }
        }

        if let Some(store) = file.store {
            if let Some(backend) = store.backend {
                self.store.backend = backend;
            }
            if let Some(root) = store.root {
                self.store.root = resolve_relative_path(base_dir, root);
            }
        }

        if let Some(frontend) = file.frontend {
            if let Some(enabled) = frontend.enabled {
                self.frontend.enabled = enabled;
            }
            if let Some(mount_path) = frontend.mount_path {
                self.frontend.mount_path = mount_path;
            }
            if let Some(static_dir) = frontend.static_dir {
                self.frontend.static_dir = Some(resolve_relative_path(base_dir, static_dir));
            }
        }
    }

    fn apply_env(&mut self, env: EnvOverrides) -> Result<(), ConfigError> {
        if let Some(listen) = env.listen {
            self.server.listen = listen;
        }
        if let Some(open_browser) = env.open_browser {
            self.server.open_browser = open_browser;
        }
        if let Some(root) = env.store_root {
            self.store.root = root;
        }
        if let Some(backend) = env.store_backend {
            self.store.backend = backend;
        }
        if let Some(enabled) = env.frontend_enabled {
            self.frontend.enabled = enabled;
        }
        if let Some(mount_path) = env.frontend_mount_path {
            self.frontend.mount_path = mount_path;
        }
        if let Some(static_dir) = env.frontend_static_dir {
            self.frontend.static_dir = Some(static_dir);
        }

        Ok(())
    }

    fn apply_cli(&mut self, cli: CliOverrides) {
        if let Some(listen) = cli.listen {
            self.server.listen = listen;
        }
        if let Some(root) = cli.store_root {
            self.store.root = root;
        }
        if let Some(backend) = cli.store_backend {
            self.store.backend = backend;
        }
    }

    fn finish(self) -> Result<AppConfig, ConfigError> {
        let mount_path = normalize_mount_path(&self.frontend.mount_path)?;

        Ok(AppConfig {
            server: self.server,
            store: self.store,
            frontend: FrontendConfig {
                enabled: self.frontend.enabled,
                mount_path,
                static_dir: self.frontend.static_dir,
            },
        })
    }
}

fn resolve_config_path(cli: &CliOverrides, env: &EnvOverrides) -> Option<PathBuf> {
    if let Some(path) = &cli.config {
        return Some(path.clone());
    }

    if let Some(path) = &env.config {
        return Some(path.clone());
    }

    let default = PathBuf::from("ss-app.toml");
    if default.is_file() {
        return Some(default);
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let packaged = exe_dir.join("ss-app.toml");
            if packaged.is_file() {
                return Some(packaged);
            }
        }
    }

    None
}

fn resolve_relative_path(base_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    }
}

fn normalize_mount_path(value: &str) -> Result<String, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::InvalidFrontendMountPath(value.to_owned()));
    }
    if !trimmed.starts_with('/') {
        return Err(ConfigError::InvalidFrontendMountPath(value.to_owned()));
    }
    if trimmed == "/" {
        return Ok("/".to_owned());
    }

    Ok(trimmed.trim_end_matches('/').to_owned())
}

fn parse_bool_env(name: &str, value: &str) -> Result<bool, ConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(ConfigError::InvalidEnvVar {
            name: name.to_owned(),
            value: value.to_owned(),
        }),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    ReadConfigFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    ParseConfigFile {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid env var {name}: {value}")]
    InvalidEnvVar { name: String, value: String },
    #[error("invalid store backend: {0}")]
    InvalidStoreBackend(String),
    #[error("frontend mount_path must start with '/' and cannot be empty: {0}")]
    InvalidFrontendMountPath(String),
}
