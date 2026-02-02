mod account;
mod daemon;
mod matrix;
mod message;
mod queue;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = if let Ok(_) = std::env::var("RUST_LOG") {
        tracing_subscriber::EnvFilter::from_default_env()
    } else {
        tracing_subscriber::EnvFilter::new("warn,nuqql_matrixd_rs=info")
    };
    tracing_subscriber::fmt::fmt()
        .with_env_filter(filter)
        .init();
    daemon::run_daemon().await
}
