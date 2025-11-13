use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

const MAX_MSG_LENGTH: u64 = 128 * 1024;

struct Client {
    from_client: mpsc::Receiver<String>,
    to_client: mpsc::Sender<String>,
}

impl Client {
    async fn send(stream: &mut WriteHalf<TcpStream>, bytes: &[u8]) -> std::io::Result<()> {
        stream.write_all(bytes).await
    }

    async fn receive(stream: &mut ReadHalf<TcpStream>) -> std::io::Result<String> {
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

    async fn handle_rx(mut stream: ReadHalf<TcpStream>, from_client: mpsc::Sender<String>) {
        loop {
            match Self::receive(&mut stream).await {
                Ok(msg) => {
                    if let Err(err) = from_client.send(msg).await {
                        println!("Error sending client message to receive channel: {err}");
                        return;
                    }
                }
                Err(err) => {
                    println!("Error receiving from client: {err}");
                    return;
                }
            }
        }
    }

    async fn handle_tx(mut stream: WriteHalf<TcpStream>, mut to_client: mpsc::Receiver<String>) {
        while let Some(msg) = to_client.recv().await {
            if let Err(err) = Self::send(&mut stream, &msg.as_bytes()).await {
                println!("Error sending to client: {err}");
                return;
            }
        }
    }

    fn new(stream: TcpStream) -> Self {
        let (from_client_tx, from_client_rx) = mpsc::channel(1);
        let (to_client_tx, to_client_rx) = mpsc::channel(1);
        let (rx, tx) = tokio::io::split(stream);
        tokio::spawn(async move { Self::handle_rx(rx, from_client_tx).await });
        tokio::spawn(async move { Self::handle_tx(tx, to_client_rx).await });
        Client {
            from_client: from_client_rx,
            to_client: to_client_tx,
        }
    }

    async fn get_message(&mut self) -> Option<String> {
        self.from_client.recv().await
    }

    async fn send_message(&mut self, msg: String) -> Result<(), mpsc::error::SendError<String>> {
        self.to_client.send(msg).await
    }
}

struct Server {
    listener: TcpListener,
}

impl Server {
    async fn listen() -> std::io::Result<Server> {
        let listener = TcpListener::bind("127.0.0.1:32000").await?;
        println!("Server listening on: {}", listener.local_addr()?);
        Ok(Server { listener })
    }

    async fn next(&self) -> std::io::Result<Client> {
        let (stream, _) = self.listener.accept().await?;
        Ok(Client::new(stream))
    }
}

async fn get_message(client: &mut Option<Client>) -> Option<Option<String>> {
    match client.as_mut() {
        Some(client) => Some(client.get_message().await),
        None => None,
    }
}

pub async fn run_server() -> std::io::Result<()> {
    let server = Server::listen().await?;

    // only one client connection is handled at the same time
    let mut client = None;
    loop {
        tokio::select! {
            // handle new client connection
            c = server.next() => match c {
                Ok(mut c) => {
                    if client.is_some() {
                        // client already connected, decline connection
                        _ = c.send_message("info: client already connected\r\n".into()).await;
                        continue;
                    }
                    if let Err(err) = c.send_message("info: Welcome to nuqql-matrixd-rs!\r\n".into()).await {
                        println!("Error sending welcome message to client: {err}");
                        continue;
                    }
                    client = Some(c);
                }
                Err(err) => {
                    // server broken?
                    println!("Error getting client: {err}");
                    return Err(err);
                }
            },

            // handle message from client
            Some(msg) = get_message(&mut client) => match msg {
                Some(msg) => {
                    print!("{msg}");
                    if let Err(err) = client.as_mut().unwrap().send_message(msg).await {
                        // client broken?
                        println!("Error sending message back to client: {err}");
                        client = None;
                        continue;
                    }
                }
                None => {
                    // client broken?
                    println!("Error getting message from client");
                    client = None;
                }
            }
        }
    }
}
