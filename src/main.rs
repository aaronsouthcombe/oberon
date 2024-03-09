use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {

    // First create a TcpListener that will listen for connections.
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    loop {
        // Here, the listener matches new connections to their Ok and Err states, and acts
        // accordingly.
        match listener.accept().await {
            // If Ok, MOVE the TcpStream created from the listener.accept into the handle_client,
            // along with the corresponding addr (plus error handling).
            Ok((client_stream, client_addr)) => { tokio::spawn(async move {
                match handle_client(client_stream, client_addr).await {
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


async fn handle_client(mut client_stream: TcpStream, client_addr: SocketAddr) -> io::Result<()> {

    // First, declare a buffer to negotiate client traffic.
    let mut buffer = vec![0; 1024];
   
    // Handle the text logic, TO-DO broker
    while let Ok(client_bytes) = client_stream.read(&mut buffer).await {
        if client_bytes == 0 {
            println!("Client closed connection");
            return Ok(());
        }

        if let Ok(client_text) = std::str::from_utf8(&buffer[..client_bytes]) {
            let trim_newline = client_text.trim_end_matches('\n');
            println!("{}: {}", client_addr.to_string(), trim_newline);
        } else {
            eprintln!("Text is not valid utf8")
        }
    }
    Ok(())
}

async fn message_broker(mut client_stream: TcpStream) {

}
