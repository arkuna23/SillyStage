use clap::Parser;
use ss_app::config::{AppConfig, Cli, CliOverrides};

#[tokio::main]
async fn main() -> Result<(), ss_app::AppError> {
    dotenvy::dotenv().ok();

    let cli = CliOverrides::from(Cli::parse());
    let config = AppConfig::load(cli)?;
    ss_app::run(config).await
}
