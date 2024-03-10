use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedWriteHalf, OwnedReadHalf};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{BrokerMessage, ClientMessage};

#[derive(Debug)]
enum ClientState {
    Registered(u32),
    Unregistered,
}


pub async fn handle_client(client_stream: TcpStream, client_addr: SocketAddr, client_sender: mpsc::UnboundedSender<BrokerMessage>) -> io::Result<()> {
    let (tcp_read, tcp_write) = client_stream.into_split();
    let (broker_sender, client_receiver) = mpsc::unbounded_channel();
   tokio::spawn( async move {
        client_tx(tcp_write, client_receiver).await;
    });
    tokio::spawn( async move {
        client_rx(tcp_read, client_sender, broker_sender).await;
    });
    Ok(())
}


async fn client_tx(mut client_stream: OwnedWriteHalf, client_receiver: mpsc::UnboundedReceiver<ClientMessage>) {
    // This function will just accept traffic from the broker and send it to the client. I wonder
    // if just making the broker send traffic through tcp straight to the client is better...?
    // At any rate, we aren't doing much inefficient stuff, cloning the tcpstream handle is 
    // really lightweight, as its just another reference to the heap living tcpstream.
    //
    let _ = client_stream.write_all(b"Welcome to Oberon, please state your username:").await;
}

async fn client_rx(mut client_stream: OwnedReadHalf, client_sender: mpsc::UnboundedSender<BrokerMessage>, broker_sender: mpsc::UnboundedSender<ClientMessage>) {
    // We move the logic of sending to the broker and interpreting incoming messages here once 
    // the client handler has finished registering.
    let mut buffer = [0; 1024];
    let mut client_id: u32 = 0;
    

    while let Ok(client_bytes) = client_stream.read(&mut buffer).await {
        match client_bytes {
            0 => {
                println!("The client appears to have closed the connection.");
                return ()
            }
            n => {
                if let Ok(client_text) = std::str::from_utf8(&buffer[..n]) {
                    let trim_newline = client_text.trim_end_matches('\n');
                    println!("client sent: {}", trim_newline);
                    // let _ = client_sender.send(BrokerMessage::Register{client_name: trim_newline.to_string(), client_chan: broker_sender.clone()});
                } else {
                    eprintln!("Text is not valid utf8");
                }

            }
        }
    }
}
