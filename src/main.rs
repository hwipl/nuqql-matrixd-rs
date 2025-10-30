mod server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello from nuqql-matrixd-rs");
    server::run_server().await
}
