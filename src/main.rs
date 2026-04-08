mod account;
mod config;
mod daemon;
mod matrix;
mod message;
mod queue;
mod server;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    let filter = tracing_subscriber::EnvFilter::new(config.loglevel);
    tracing_subscriber::fmt::fmt()
        .with_env_filter(filter)
        .init();
    daemon::run_daemon().await
}
