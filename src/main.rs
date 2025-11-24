mod message;
mod server;

use crate::message::Message;
use server::{Client, Server};
use tokio::sync::mpsc;

async fn get_message(client: &mut Option<Client>) -> Option<Option<Message>> {
    match client.as_mut() {
        Some(client) => Some(client.get_message().await),
        None => None,
    }
}

async fn handle_message(
    client: &mut Client,
    msg: Message,
) -> Result<(), mpsc::error::SendError<Message>> {
    print!("{msg}");
    match msg {
        Message::Help => {
            let reply = Message::Info {
                message: "help text".into(),
            };
            return client.send_message(reply).await;
        }
        _ => (),
    }
    client.send_message(msg).await
}

async fn run_server() -> std::io::Result<()> {
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
                        // FIXME
                        _ = c.send_message(Message::Info{message: "info: client already connected\r\n".into()}).await;
                        continue;
                    }
                    // FIXME
                    if let Err(err) = c.send_message(Message::Info{message: "info: Welcome to nuqql-matrixd-rs!\r\n".into()}).await {
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
                    if let Err(err) = handle_message(client.as_mut().unwrap(), msg).await {
                        // client broken?
                        println!("Error handling message: {err}");
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
#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello from nuqql-matrixd-rs");
    run_server().await
}
