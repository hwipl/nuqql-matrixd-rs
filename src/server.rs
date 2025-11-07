use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

const MAX_MSG_LENGTH: u64 = 128 * 1024;

struct Client {
    stream: TcpStream,
}

impl Client {
    async fn send(stream: &mut TcpStream, bytes: &[u8]) -> std::io::Result<()> {
        stream.write_all(bytes).await
    }

    async fn receive(stream: &mut TcpStream) -> std::io::Result<String> {
        let mut buf = String::new();
        let mut stream = BufReader::new(stream.take(MAX_MSG_LENGTH));
        loop {
            if buf.ends_with("\r\n") {
                return Ok(buf);
            }
            if stream.read_line(&mut buf).await? == 0 {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }
        }
    }

    async fn handle_client(
        &mut self,
        from_client: mpsc::Sender<String>,
        mut to_client: mpsc::Receiver<String>,
    ) {
        if let Err(err) = Self::send(&mut self.stream, b"Welcome to nuqql-matrixd-rs!\r\n").await {
            println!("Error sending to client: {err}");
        }
        loop {
            tokio::select! {
                msg = Self::receive(&mut self.stream) => match msg {
                    Ok(msg) => {
                        if let Err(err) = from_client.send(msg).await {
                            println!("Error sending client message to receive channel: {err}");
                            return;
                        }
                    }
                    Err(err) => {
                        println!("Error receiving from client: {err}");
                        return
                    }
                },
                Some(msg) = to_client.recv() => {
                    if let Err(err) = Self::send(&mut self.stream, &msg.as_bytes()).await {
                        println!("Error sending to client: {err}");
                        return;
                    }
                }
            }
        }
    }

    fn new(stream: TcpStream) -> Self {
        Client { stream: stream }
    }
}

pub async fn run_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:32000").await?;

    println!("Server listening on: {}", listener.local_addr()?);

    loop {
        let (stream, _) = listener.accept().await?;

        // only one client connection is allowed at the same time
        let (from_client_tx, mut from_client_rx) = mpsc::channel(1);
        let (to_client_tx, to_client_rx) = mpsc::channel(1);
        let mut client = Client::new(stream);
        tokio::spawn(async move { client.handle_client(from_client_tx, to_client_rx).await });
        while let Some(msg) = from_client_rx.recv().await {
            print!("{msg}");
            if let Err(err) = to_client_tx.send(msg).await {
                println!("Error sending message to send channel: {err}");
                break;
            }
        }
    }
}
