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
    let (mut tcp_read, tcp_write) = client_stream.into_split();
    let (broker_sender, mut client_receiver) = mpsc::unbounded_channel();
    let mut buffer = [0; 1024];
    let mut client_id: u32 = 0;


    match tcp_read.read(&mut buffer).await {
        Ok(0) => {
            println!("The client has closed the connection.");
        }
        Ok(n) => {
            if let Ok(client_text) = std::str::from_utf8(&buffer[..n]) {
                let trim_newline = client_text.trim_end_matches('\n');
                println!("cli_hand: {}", trim_newline);
                let _ = client_sender.send(BrokerMessage::Register{client_name: trim_newline.to_string(), client_chan: broker_sender.clone()});
                while let Some(message) = client_receiver.recv().await {
                    match message {
                        ClientMessage::ClientId(new_id) => {
                            client_id = new_id;
                            break
                        }
                        
                        _ => {eprintln!("Error collecting new registration");}
                    }
                }
            } else {
                eprintln!("Text is not valid utf8");
            }

        }

        _ => {println!("Unknown error in registration");}
    }

    tokio::spawn( async move {
        client_tx(tcp_write, client_receiver).await;
    });
    tokio::spawn( async move {
        client_rx(tcp_read, client_sender, broker_sender, client_id).await;
    });
    Ok(())
}


async fn client_tx(mut client_stream: OwnedWriteHalf, mut client_receiver: mpsc::UnboundedReceiver<ClientMessage>) {
    // This function will just accept traffic from the broker and send it to the client. I wonder
    // if just making the broker send traffic through tcp straight to the client is better...?
    // At any rate, we aren't doing much inefficient stuff, cloning the tcpstream handle is 
    // really lightweight, as its just another reference to the heap living tcpstream.
    //
    while let Some(message) = client_receiver.recv().await {
        match message {
            ClientMessage::Message { content } => {
                let _ = client_stream.write_all(content.as_bytes()).await;
            }
            _ => {println!("Unhandled behaviour on client transmitter");}
        }
    }
}

async fn client_rx(mut client_stream: OwnedReadHalf, client_sender: mpsc::UnboundedSender<BrokerMessage>, broker_sender: mpsc::UnboundedSender<ClientMessage>, client_id: u32) {
    // We move the logic of sending to the broker and interpreting incoming messages here once 
    // the client handler has finished registering.
    let mut buffer = [0; 1024];

    loop {
        match client_stream.read(&mut buffer).await {
            Ok(0) => {
                println!("The client has closed the connection.");
                return ()
            }

            Ok(n) => {
                if let Ok(client_text) = std::str::from_utf8(&buffer[..n]) {
                    // let trim_newline = client_text.trim_end_matches('\n');
                    println!("cli_msg: {}", client_text);
                    if let Some((dest_name, dest_msg)) = parse_message(client_text) {
                        let _ = client_sender.send(BrokerMessage::Message(client_id, dest_name, dest_msg.to_string()));
                    }
                } else {
                    eprintln!("Text is not valid utf8");
                }
            }
            
            _ => {println!("Unknown error in msg loop");}
        }
    }
}

fn parse_message(input: &str) -> Option<(String, &str)> {
    let re = regex::Regex::new(r"^%%(.*?)%%(.*)").unwrap();
    re.captures(input).map(|caps| {
        (caps[1].to_string(), caps.get(2).map_or("", |m| m.as_str()))
    })
}
