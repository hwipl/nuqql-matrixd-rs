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

    async fn send_message(&mut self, msg: String) -> Result<(), impl std::error::Error> {
        self.to_client.send(msg).await
    }
}

struct Server {
    clients_tx: mpsc::Sender<Client>,
    clients_rx: mpsc::Receiver<Client>,
}

impl Server {
    async fn handle_clients(listener: TcpListener, clients_tx: mpsc::Sender<Client>) {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let client = Client::new(stream);
                    if let Err(err) = clients_tx.send(client).await {
                        println!("{err}");
                        return;
                    }
                }
                Err(err) => {
                    println!("Error accepting client connection: {err}");
                    return;
                }
            }
        }
    }

    fn new() -> Self {
        let (clients_tx, clients_rx) = mpsc::channel(1);
        Server {
            clients_tx: clients_tx,
            clients_rx: clients_rx,
        }
    }

    async fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:32000").await?;

        println!("Server listening on: {}", listener.local_addr()?);

        let clients_tx = self.clients_tx.clone();
        tokio::spawn(async move { Self::handle_clients(listener, clients_tx).await });
        Ok(())
    }

    async fn get_client(&mut self) -> Option<Client> {
        self.clients_rx.recv().await
    }
}

pub async fn run_server() -> std::io::Result<()> {
    let mut server = Server::new();
    server.run().await?;

    // only one client connection is allowed at the same time
    while let Some(mut client) = server.get_client().await {
        if let Err(err) = client
            .send_message("Welcome to nuqql-matrixd-rs!\r\n".into())
            .await
        {
            println!("Error sending welcome message to client: {err}");
            continue;
        }
        while let Some(msg) = client.get_message().await {
            print!("{msg}");
            if let Err(err) = client.send_message(msg).await {
                println!("Error sending message back to client: {err}");
                break;
            }
        }
    }
    Ok(())
}
