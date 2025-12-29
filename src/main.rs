mod account;
mod daemon;
mod message;
mod queue;
mod server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    daemon::run_daemon().await
}
