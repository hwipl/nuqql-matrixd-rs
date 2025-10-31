use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn send(stream: &mut TcpStream, bytes: &[u8]) -> std::io::Result<()> {
    stream.write_all(bytes).await
}

async fn receive(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut buf = String::new();
    let mut stream = BufReader::new(stream);
    loop {
        let num_bytes = stream.read_line(&mut buf).await?;
        if num_bytes == 0 {
            break;
        }
        let mut r = false;
        if let Some((_, '\r')) = buf.char_indices().nth_back(1) {
            r = true
        };
        let mut n = false;
        if let Some((_, '\n')) = buf.char_indices().nth_back(0) {
            n = true
        }
        if r && n {
            break;
        }
    }
    return Ok(buf);
}

async fn handle_client(mut stream: TcpStream) {
    send(&mut stream, b"Welcome to nuqql-matrixd-rs!\r\n")
        .await
        .unwrap();
    let msg = receive(&mut stream).await.unwrap();
    println!("{}", msg);
    let (mut reader, mut writer) = stream.split();
    if let Err(e) = tokio::io::copy(&mut reader, &mut writer).await {
        println!("Error reading from client: {}", e);
    }
}

pub async fn run_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:32000").await?;

    println!("Server listening on: {}", listener.local_addr()?);

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_client(stream).await;
        });
    }
}
