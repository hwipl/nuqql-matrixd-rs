use tokio::net::{TcpListener, TcpStream};

async fn handle_client(mut stream: TcpStream) {
    let (mut reader, mut writer) = stream.split();
    if let Err(e) = tokio::io::copy(&mut reader, &mut writer).await {
        println!("Error reading from client: {}", e);
    }
}

pub async fn run_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;

    println!("Server listening on: {}", listener.local_addr()?);

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_client(stream).await;
        });
    }
}
