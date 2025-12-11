use crate::message::Message;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

pub const LISTEN_ADDRESS: &str = "127.0.0.1:32000";
pub const MAX_MSG_LENGTH: u64 = 128 * 1024;
pub const SEND_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Config {
    pub listen_address: String,
    pub max_msg_length: u64,
    pub send_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen_address: LISTEN_ADDRESS.into(),
            max_msg_length: MAX_MSG_LENGTH,
            send_timeout: SEND_TIMEOUT,
        }
    }
}

pub struct Client {
    from_client: mpsc::Receiver<Message>,
    to_client: mpsc::Sender<Message>,
}

impl Client {
    async fn send(
        stream: &mut WriteHalf<TcpStream>,
        timeout: Duration,
        bytes: &[u8],
    ) -> std::io::Result<()> {
        tokio::time::timeout(timeout, stream.write_all(bytes))
            .await
            .unwrap_or(Err(std::io::ErrorKind::TimedOut.into()))
    }

    async fn receive(
        stream: &mut ReadHalf<TcpStream>,
        max_msg_length: u64,
    ) -> std::io::Result<String> {
        let mut buf = String::new();
        let mut stream = BufReader::new(stream.take(max_msg_length));
        loop {
            if buf.ends_with("\r\n") {
                return Ok(buf);
            }
            if stream.read_line(&mut buf).await? == 0 {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }
        }
    }

    async fn handle_rx(
        mut stream: ReadHalf<TcpStream>,
        from_client: mpsc::Sender<Message>,
        to_client: mpsc::Sender<Message>,
        max_msg_length: u64,
    ) {
        loop {
            tokio::select! {
                // receive message and forward it to receiver
                msg = Self::receive(&mut stream, max_msg_length) => match msg {
                    Ok(msg) => {
                        let Ok(msg) = msg.parse() else { continue };
                        if let Err(err) = from_client.send(msg).await {
                            println!("Error sending client message to receive channel: {err}");
                            return;
                        }
                    }
                    Err(err) => {
                        println!("Error receiving from client: {err}");
                        return;
                    }
                },

                // abort if there is no receiver
                _ = from_client.closed() => return,

                // abort if tx handler closed
                _ = to_client.closed() => return,
            }
        }
    }

    async fn handle_tx(
        mut stream: WriteHalf<TcpStream>,
        mut to_client: mpsc::Receiver<Message>,
        send_timeout: Duration,
    ) {
        while let Some(msg) = to_client.recv().await {
            let msg = msg.to_string();
            if let Err(err) = Self::send(&mut stream, send_timeout, &msg.as_bytes()).await {
                println!("Error sending to client: {err}");
                return;
            }
        }
    }

    fn new(stream: TcpStream, max_msg_length: u64, send_timeout: Duration) -> Self {
        let (from_client_tx, from_client_rx) = mpsc::channel(1);
        let (to_client_tx, to_client_rx) = mpsc::channel(1);
        let to_client_tx_check = to_client_tx.clone();
        let (rx, tx) = tokio::io::split(stream);
        tokio::spawn(async move {
            Self::handle_rx(rx, from_client_tx, to_client_tx_check, max_msg_length).await
        });
        tokio::spawn(async move { Self::handle_tx(tx, to_client_rx, send_timeout).await });
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
    config: Config,
    listener: TcpListener,
}

impl Server {
    pub async fn listen(config: Config) -> std::io::Result<Server> {
        let listener = TcpListener::bind(&config.listen_address).await?;
        Ok(Server { config, listener })
    }

    pub fn listen_address(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    pub async fn next(&self) -> std::io::Result<Client> {
        let (stream, _) = self.listener.accept().await?;
        Ok(Client::new(
            stream,
            self.config.max_msg_length,
            self.config.send_timeout,
        ))
    }
}
