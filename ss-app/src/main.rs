use clap::Parser;
use ss_app::config::{AppConfig, Cli, CliOverrides};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), ss_app::AppError> {
    dotenvy::dotenv().ok();
    init_logging();

    let cli = CliOverrides::from(Cli::parse());
    let config = AppConfig::load(cli)?;
    ss_app::run(config).await
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(concat!(
            "info,",
            "ss_app=info,",
            "ss_server=info,",
            "ss_handler=info,",
            "ss_engine=info,",
            "ss_agents=info,",
            "ss_llm_api=info,",
            "ss_store=info,",
            "ss_protocol=info,",
            "ss_story=info,",
            "ss_state=info"
        ))
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .init();
}
