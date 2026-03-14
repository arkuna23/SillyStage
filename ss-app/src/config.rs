use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use engine::AgentApiIds;
use serde::Deserialize;
pub use store::LlmProvider;

#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub store: StoreConfig,
    pub frontend: FrontendConfig,
    pub llm: LlmConfig,
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

#[derive(Debug, Clone, PartialEq)]
pub struct LlmConfig {
    pub apis: BTreeMap<String, LlmApiConfig>,
    pub defaults: Option<AgentApiIds>,
    pub default_config: Option<DefaultLlmConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmApiConfig {
    pub provider: LlmProvider,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLlmConfig {
    pub provider: LlmProvider,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
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
    pub store_root: Option<PathBuf>,
    pub store_backend: Option<StoreBackend>,
    pub frontend_enabled: Option<bool>,
    pub frontend_mount_path: Option<String>,
    pub frontend_static_dir: Option<PathBuf>,
    pub default_openai_base_url: Option<String>,
    pub default_openai_api_key: Option<String>,
    pub default_openai_model: Option<String>,
    pub default_openai_temperature: Option<f32>,
    pub default_openai_max_tokens: Option<u32>,
}

impl EnvOverrides {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            config: env::var_os("SS_APP_CONFIG").map(PathBuf::from),
            listen: env::var("SS_APP_LISTEN").ok(),
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
            default_openai_base_url: env::var("LLM_API_BASE").ok(),
            default_openai_api_key: env::var("LLM_API_KEY").ok(),
            default_openai_model: env::var("LLM_API_MODEL").ok(),
            default_openai_temperature: env::var("LLM_API_TEMPERATURE")
                .ok()
                .map(|value| parse_f32_env("LLM_API_TEMPERATURE", &value))
                .transpose()?,
            default_openai_max_tokens: env::var("LLM_API_MAX_TOKENS")
                .ok()
                .map(|value| parse_u32_env("LLM_API_MAX_TOKENS", &value))
                .transpose()?,
        })
    }
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    server: Option<FileServerConfig>,
    store: Option<FileStoreConfig>,
    frontend: Option<FileFrontendConfig>,
    llm: Option<FileLlmConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct FileServerConfig {
    listen: Option<String>,
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

#[derive(Debug, Default, Deserialize)]
struct FileLlmConfig {
    apis: Option<BTreeMap<String, FileLlmApiConfig>>,
    defaults: Option<AgentApiIds>,
    default_config: Option<FileDefaultLlmConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct FileLlmApiConfig {
    provider: Option<LlmProvider>,
    base_url: String,
    api_key: String,
    model: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct FileDefaultLlmConfig {
    provider: Option<LlmProvider>,
    base_url: String,
    api_key: String,
    model: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Debug, Clone)]
struct ResolvedConfig {
    server: ServerConfig,
    store: StoreConfig,
    frontend: FrontendConfig,
    llm_apis: BTreeMap<String, LlmApiConfig>,
    llm_defaults: Option<AgentApiIds>,
    llm_default_config: Option<DefaultLlmConfig>,
}

impl Default for ResolvedConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                listen: "127.0.0.1:8080".to_owned(),
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
            llm_apis: BTreeMap::new(),
            llm_defaults: None,
            llm_default_config: None,
        }
    }
}

impl ResolvedConfig {
    fn apply_file(&mut self, file: FileConfig, base_dir: &Path) {
        if let Some(server) = file.server
            && let Some(listen) = server.listen
        {
            self.server.listen = listen;
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

        if let Some(llm) = file.llm {
            if let Some(apis) = llm.apis {
                for (api_id, config) in apis {
                    self.llm_apis.insert(
                        api_id,
                        LlmApiConfig {
                            provider: config.provider.unwrap_or(LlmProvider::OpenAi),
                            base_url: config.base_url,
                            api_key: config.api_key,
                            model: config.model,
                            temperature: config.temperature,
                            max_tokens: config.max_tokens,
                        },
                    );
                }
            }
            if let Some(defaults) = llm.defaults {
                self.llm_defaults = Some(defaults);
            }
            if let Some(default_config) = llm.default_config {
                self.llm_default_config = Some(DefaultLlmConfig {
                    provider: default_config.provider.unwrap_or(LlmProvider::OpenAi),
                    base_url: default_config.base_url,
                    api_key: default_config.api_key,
                    model: default_config.model,
                    temperature: default_config.temperature,
                    max_tokens: default_config.max_tokens,
                });
            }
        }
    }

    fn apply_env(&mut self, env: EnvOverrides) -> Result<(), ConfigError> {
        if let Some(listen) = env.listen {
            self.server.listen = listen;
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

        match (
            env.default_openai_base_url,
            env.default_openai_api_key,
            env.default_openai_model,
            env.default_openai_temperature,
            env.default_openai_max_tokens,
        ) {
            (Some(base_url), Some(api_key), Some(model), temperature, max_tokens) => {
                self.llm_default_config = Some(DefaultLlmConfig {
                    provider: LlmProvider::OpenAi,
                    base_url,
                    api_key,
                    model,
                    temperature,
                    max_tokens,
                });
            }
            (None, None, None, None, None) => {}
            _ => return Err(ConfigError::IncompleteEnvDefaultLlmConfig),
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
        let defaults = resolve_default_api_ids(&self.llm_apis, self.llm_defaults);

        Ok(AppConfig {
            server: self.server,
            store: self.store,
            frontend: FrontendConfig {
                enabled: self.frontend.enabled,
                mount_path,
                static_dir: self.frontend.static_dir,
            },
            llm: LlmConfig {
                apis: self.llm_apis,
                defaults,
                default_config: self.llm_default_config,
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
    default.is_file().then_some(default)
}

fn resolve_relative_path(base_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    }
}

fn resolve_default_api_ids(
    apis: &BTreeMap<String, LlmApiConfig>,
    configured: Option<AgentApiIds>,
) -> Option<AgentApiIds> {
    if let Some(defaults) = configured {
        return Some(defaults);
    }

    if let Some(default_api_id) = apis.get_key_value("default").map(|(key, _)| key.clone()) {
        return Some(repeat_api_id(&default_api_id));
    }

    if apis.len() == 1 {
        let api_id = apis.keys().next().expect("checked len").clone();
        return Some(repeat_api_id(&api_id));
    }

    None
}

fn repeat_api_id(api_id: &str) -> AgentApiIds {
    AgentApiIds {
        planner_api_id: api_id.to_owned(),
        architect_api_id: api_id.to_owned(),
        director_api_id: api_id.to_owned(),
        actor_api_id: api_id.to_owned(),
        narrator_api_id: api_id.to_owned(),
        keeper_api_id: api_id.to_owned(),
        replyer_api_id: api_id.to_owned(),
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

fn parse_f32_env(name: &str, value: &str) -> Result<f32, ConfigError> {
    value
        .trim()
        .parse::<f32>()
        .map_err(|_| ConfigError::InvalidEnvVar {
            name: name.to_owned(),
            value: value.to_owned(),
        })
}

fn parse_u32_env(name: &str, value: &str) -> Result<u32, ConfigError> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| ConfigError::InvalidEnvVar {
            name: name.to_owned(),
            value: value.to_owned(),
        })
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
    #[error(
        "LLM_API_BASE, LLM_API_KEY, and LLM_API_MODEL must all be set together to override the default llm config"
    )]
    IncompleteEnvDefaultLlmConfig,
}
