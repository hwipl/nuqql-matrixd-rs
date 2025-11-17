use crate::message::Message;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

const MAX_MSG_LENGTH: u64 = 128 * 1024;

pub struct Client {
    from_client: mpsc::Receiver<Message>,
    to_client: mpsc::Sender<Message>,
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

    async fn handle_rx(mut stream: ReadHalf<TcpStream>, from_client: mpsc::Sender<Message>) {
        loop {
            match Self::receive(&mut stream).await {
                Ok(msg) => {
                    if let Err(err) = from_client.send(msg.into()).await {
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

    async fn handle_tx(mut stream: WriteHalf<TcpStream>, mut to_client: mpsc::Receiver<Message>) {
        while let Some(msg) = to_client.recv().await {
            let msg = String::from(msg);
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

    pub async fn get_message(&mut self) -> Option<Message> {
        self.from_client.recv().await
    }

    pub async fn send_message(
        &mut self,
        msg: Message,
    ) -> Result<(), mpsc::error::SendError<Message>> {
        self.to_client.send(msg).await
    }
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn listen() -> std::io::Result<Server> {
        let listener = TcpListener::bind("127.0.0.1:32000").await?;
        println!("Server listening on: {}", listener.local_addr()?);
        Ok(Server { listener })
    }

    pub async fn next(&self) -> std::io::Result<Client> {
        let (stream, _) = self.listener.accept().await?;
        Ok(Client::new(stream))
    }
}
