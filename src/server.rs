use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn send(stream: &mut TcpStream, bytes: &[u8]) -> std::io::Result<()> {
    stream.write_all(bytes).await
}

async fn receive(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut buf = String::new();
    let mut stream = BufReader::new(stream);
    loop {
        if buf.ends_with("\r\n") {
            return Ok(buf);
        }
        if stream.read_line(&mut buf).await? == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
    }
}

async fn handle_client(mut stream: TcpStream) {
    if let Err(err) = send(&mut stream, b"Welcome to nuqql-matrixd-rs!\r\n").await {
        println!("Error sending to client: {err}");
    }
    loop {
        match receive(&mut stream).await {
            Ok(msg) => print!("{msg}"),
            Err(err) => {
                println!("Error receiving from client: {err}");
                return;
            }
        }
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
