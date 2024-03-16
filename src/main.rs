use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::io;

use crate::client::handle_client;

mod broker;
mod client;

enum BrokerMessage {
    Register { client_name: String, client_chan: mpsc::UnboundedSender<ClientMessage>},
    Message(u32, String, String)
}

enum ClientMessage {
    ClientId(u32),
    Message { content: String },
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
            Ok((client_stream, _client_addr)) => { tokio::spawn(async move {
                match handle_client(client_stream, client_sender).await {
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
