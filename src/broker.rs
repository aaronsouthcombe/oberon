use tokio::sync::mpsc;
use crate::{BrokerMessage, ClientMessage};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

static CLIENT_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

pub async fn message_broker(mut globalbroker_reciever: mpsc::UnboundedReceiver<BrokerMessage>) {
    let mut clients:  HashMap<u32, mpsc::UnboundedSender<ClientMessage>> = HashMap::new();
    let mut client_names: HashMap<String, u32> = HashMap::new();

    while let Some(message) = globalbroker_reciever.recv().await {
        match message {
            BrokerMessage::Register{ client_name, client_chan }  => {
                let new_id = CLIENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                println!("New client registered, ID: {}, Name: {}", new_id, client_name);
                let _ = client_chan.send(ClientMessage::ClientId(new_id));
                clients.insert(new_id, client_chan);
                client_names.insert(client_name, new_id);
                    ()
            }
            BrokerMessage::Message(id, dst_name, msg) => {
                let &dst_id = client_names.get(&dst_name).unwrap();
                let sender = clients.get(&dst_id);
                sender.unwrap().send(ClientMessage::Message { content: msg.to_string() }).unwrap();
                println!("from {} to {}: {}", id, dst_name, msg);
            }
        };
    }
}
