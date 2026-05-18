use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{net::{TcpListener, TcpStream}, sync::{broadcast, Mutex}};
use tokio_websockets::{Message, ServerBuilder, WebsocketStream};

// Struktur JSON untuk berkomunikasi dengan Frontend Yew
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct MessageData {
    from: String,
    message: String,
}

// Global State untuk menyimpan daftar IP dan Username yang terhubung
type SharedUsers = Arc<Mutex<HashMap<SocketAddr, String>>>;

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebsocketStream<TcpStream>,
    bcast_tx: broadcast::Sender<String>,
    users: SharedUsers,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            // MENERIMA PESAN DARI KLIEN
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            let text = msg.as_text().unwrap();
                            
                            // Parse JSON dari Yew Frontend
                            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                                match ws_msg.message_type {
                                    MsgTypes::Register => {
                                        if let Some(username) = ws_msg.data {
                                            // Simpan user baru ke memory
                                            let mut users_lock = users.lock().await;
                                            users_lock.insert(addr, username);
                                            let users_list: Vec<String> = users_lock.values().cloned().collect();
                                            drop(users_lock); // Lepas lock secepatnya

                                            // Broadcast update daftar user ke semua orang
                                            let response = WebSocketMessage {
                                                message_type: MsgTypes::Users,
                                                data_array: Some(users_list),
                                                data: None,
                                            };
                                            let _ = bcast_tx.send(serde_json::to_string(&response).unwrap());
                                        }
                                    },
                                    MsgTypes::Message => {
                                        if let Some(msg_text) = ws_msg.data {
                                            let users_lock = users.lock().await;
                                            if let Some(username) = users_lock.get(&addr) {
                                                // Bungkus chat atau data Arena dengan MessageData (seperti Node.js)
                                                let msg_data = MessageData {
                                                    from: username.clone(),
                                                    message: msg_text,
                                                };
                                                
                                                let response = WebSocketMessage {
                                                    message_type: MsgTypes::Message,
                                                    data: Some(serde_json::to_string(&msg_data).unwrap()),
                                                    data_array: None,
                                                };
                                                let _ = bcast_tx.send(serde_json::to_string(&response).unwrap());
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => break, // Koneksi terputus
                }
            }
            // MENERUSKAN BROADCAST KE KLIEN INI
            msg = bcast_rx.recv() => {
                if let Ok(msg_text) = msg {
                    ws_stream.send(Message::text(msg_text)).await?;
                }
            }
        }
    }

    // Jika kode sampai sini, berarti user menutup tab browser.
    // Hapus user dari HashMap dan broadcast daftar user terbaru
    let mut users_lock = users.lock().await;
    users_lock.remove(&addr);
    let users_list: Vec<String> = users_lock.values().cloned().collect();
    drop(users_lock);

    let response = WebSocketMessage {
        message_type: MsgTypes::Users,
        data_array: Some(users_list),
        data: None,
    };
    let _ = bcast_tx.send(serde_json::to_string(&response).unwrap());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (bcast_tx, _) = broadcast::channel(100);
    // Inisialisasi state global Thread-Safe
    let users: SharedUsers = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Rust WebSocket Server for YewChat is live on port 8080!");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("User connected from: {addr:?}");
        
        let bcast_tx = bcast_tx.clone();
        let users = users.clone();
        
        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await.unwrap();
            if let Err(e) = handle_connection(addr, ws_stream, bcast_tx, users).await {
                println!("Connection error from {addr:?}: {e}");
            }
        });
    }
}