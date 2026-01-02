mod account;
mod daemon;
mod matrix;
mod message;
mod queue;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    daemon::run_daemon().await
}
