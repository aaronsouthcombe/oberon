use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod broker;

enum BrokerMessage {
    Register { client_name: String, client_chan: mpsc::UnboundedSender<ClientMessage>},
    Unregister(Client),
    Command { command: String },
    Message { content: String },
}

#[derive(Debug)]
enum ClientState {
    Registered(u32),
    Unregistered,
}

enum ClientMessage {
    ClientId(u32),
    Message { content: String },
    CommandResponse { command_response: String },
}

struct Client {
    id: u32,
    sender: mpsc::UnboundedSender<ClientMessage>,
}

#[tokio::main]
async fn main() -> io::Result<()> {

    // First create a TcpListener that will listen for connections.
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    
    let (globalbroker_sender, globalbroker_reciever) = mpsc::unbounded_channel::<BrokerMessage>();

    tokio::spawn(async move {
        broker::message_broker(globalbroker_reciever).await;
    });

    loop {
        // Here, the listener matches new connections to their Ok and Err states, and acts
        // accordingly.
        let client_sender = globalbroker_sender.clone();
        match listener.accept().await {
            // If Ok, MOVE the TcpStream created from the listener.accept into the handle_client,
            // along with the corresponding addr (plus error handling).
            Ok((client_stream, client_addr)) => { tokio::spawn(async move {
                match handle_client(client_stream, client_addr, client_sender).await {
                    Ok(_) => println!("Handled successfully"),
                    Err(_) => eprintln!("Client handle failed")
                }
            });
            ()},
            Err(e) => { println!("{:?}", e); break}
        };
    }
    Ok(())
}


async fn handle_client(mut client_stream: TcpStream, client_addr: SocketAddr, client_sender: mpsc::UnboundedSender<BrokerMessage>) -> io::Result<()> {

    let mut state = ClientState::Unregistered;
    // First, declare a buffer to negotiate client traffic.
    let mut buffer = vec![0; 1024];
    let (broker_sender, mut client_receiver) = mpsc::unbounded_channel();
   
    // Handle the text logic, TO-DO broker
    while let Ok(client_bytes) = client_stream.read(&mut buffer).await {
        if client_bytes == 0 {
            println!("Client closed connection");
            return Ok(());
        }

        if let Ok(client_text) = std::str::from_utf8(&buffer[..client_bytes]) {
            let trim_newline = client_text.trim_end_matches('\n');
            match state {
                ClientState::Unregistered => {
                    let _ = client_sender.send(BrokerMessage::Register{client_name: trim_newline.to_string(), client_chan: broker_sender.clone()});
                    while let Some(message) = client_receiver.recv().await {
                        match message {
                            ClientMessage::ClientId(new_id) => {
                                state = ClientState::Registered(new_id);
                                println!("state {:?}", state);
                                ()
                            }
                            _ => {}
                        }
                    }
                                
                } 
                ClientState::Registered(new_id) => {}
            };
                                       
        } else {
            eprintln!("Text is not valid utf8")
        }
    }
    Ok(())
}
