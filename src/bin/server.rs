use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebsocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebsocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ws_stream.send(Message::text("Welcome to chat! Type a message".to_string())).await?;
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            // Menerima pesan dari klien
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            let text = msg.as_text().unwrap();
                            println!("From client {addr:?} {text:?}");
                            bcast_tx.send(text.into())?;
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }
            // Menerima pesan dari klien lain via broadcast channel
            msg = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);

    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("listening on port 2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();
        
        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await.unwrap();
            if let Err(e) = handle_connection(addr, ws_stream, bcast_tx).await {
                println!("Error processing connection: {e}");
            }
        });
    }
}