use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedWriteHalf, OwnedReadHalf};
use tokio::sync::mpsc;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{BrokerMessage, ClientMessage};
/* This module is the client handler. It serves three purposes. The first, is splitting the
 * tcpstream into an rx and tx channel. Then, it creates the necessary mpsc channel for the client
 * handlers to be able to send messages to the broker uniquely, and be able to receive messages
 * from the broker. This is pretty cool, due to the fact that you can selectively broadcast
 * messages depending on channel. Once these are created, it performs its final function,
 * registering the user via the mpsc channel. This is essentially just telling the broker that
 * there is a new client with name X and channel Y, and it expects ID Z back. After all this, it
 * spawns the tx and rx handler which will listen independently and run the logic to indefinately
 * exchange messages with their client and the broker.
*/
pub async fn handle_client(client_stream: TcpStream, client_sender: mpsc::UnboundedSender<BrokerMessage>) -> io::Result<()> {
    let (mut tcp_read, tcp_write) = client_stream.into_split();
    let (broker_sender, mut client_receiver) = mpsc::unbounded_channel();
    let mut buffer = [0; 1024];
    let mut client_id: u32 = 0;


    // Simple registration logic, use the BrokerMessage::Register type to send the info to the
    // broker for it to register this client, or act accordingly.
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

    // Spawn and move. All values necessary get moved into their functions.
    tokio::spawn( async move {
        client_tx(tcp_write, client_receiver).await;
    });
    tokio::spawn( async move {
        client_rx(tcp_read, client_sender, client_id).await;
    });
    Ok(())
}


async fn client_tx(mut client_stream: OwnedWriteHalf, mut client_receiver: mpsc::UnboundedReceiver<ClientMessage>) {

    // Here we just relay messages from the broker to the client's tcp stream.
    while let Some(message) = client_receiver.recv().await {
        match message {
            ClientMessage::Message { content } => {
                let _ = client_stream.write_all(content.as_bytes()).await;
            }
            _ => {println!("Unhandled behaviour on client transmitter");}
        }
    }
}

async fn client_rx(mut client_stream: OwnedReadHalf, client_sender: mpsc::UnboundedSender<BrokerMessage>, client_id: u32) {

    // Given a registered client, we parse the dest client id from the incoming message, then
    // forward the data to the broker through its mpsc channel.
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


// This function is really just so I can move the somewhat ugly regex away from the handler.
fn parse_message(input: &str) -> Option<(String, &str)> {
    let re = regex::Regex::new(r"^%%(.*?)%%(.*)").unwrap();
    re.captures(input).map(|caps| {
        (caps[1].to_string(), caps.get(2).map_or("", |m| m.as_str()))
    })
}
