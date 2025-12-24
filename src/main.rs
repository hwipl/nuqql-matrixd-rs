mod account;
mod daemon;
mod message;
mod queue;
mod server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    daemon::run_daemon().await
}
