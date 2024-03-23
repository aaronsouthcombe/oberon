use tokio::sync::mpsc;
use crate::{BrokerMessage, ClientMessage};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

static CLIENT_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

/* This code is the definition of the communications broker. It's function is to handle
 * registration via maintaining a list of users and their info, and exchanging messages between
 * clients via their corresponding channels. This is based on the value given via the mpsc channel
 * for every client handling instance.
*/
pub async fn message_broker(mut globalbroker_reciever: mpsc::UnboundedReceiver<BrokerMessage>) {
    let mut clients:  HashMap<u32, mpsc::UnboundedSender<ClientMessage>> = HashMap::new();
    let mut client_names: HashMap<String, u32> = HashMap::new();

    // In this case, we handle register and message.
    while let Some(message) = globalbroker_reciever.recv().await {
        match message {
            BrokerMessage::Register{ client_name, client_chan }  => {

                // Increment id counter and insert new data into hashmaps, then return the new id
                // to the client's function so it can give it to the rx task and therefore it can
                // identify itself properly when it sends messages to the broker.
                let new_id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                println!("New client registered, ID: {}, Name: {}", new_id, client_name);
                let _ = client_chan.send(ClientMessage::ClientId(new_id));
                clients.insert(new_id, client_chan);
                client_names.insert(client_name, new_id);
                    ()
            }
            BrokerMessage::Message(id, dst_name, msg) => {

                // Simple code to send the message to the corresponding client.
                println!("from {} to {}: {}", id, dst_name, msg);
                for (key, value) in &client_names {
                    println!("{}: {}", key, value);
                }
                let &dst_id = client_names.get(&dst_name).unwrap();
                println!("Toid {}", dst_id);
                let sender = clients.get(&dst_id);
                sender.unwrap().send(ClientMessage::Message { content: msg.to_string() }).unwrap();
            }
        };
    }
}
