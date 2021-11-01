use std::net::*;

use std::{
    collections::{HashMap, VecDeque},
    net::Ipv4Addr,
    str::from_utf8,
    time::Duration,
};

use anyhow::Result;
use bytes::Bytes;
use qp2p::{Config, ConnId, Endpoint};
use simple_logger::SimpleLogger;
use tokio::{
    select,
    sync::mpsc::{self, Receiver, Sender},
};

const ACK: &str = "###ack###";

#[derive(Default, Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
struct XId(pub [u8; 32]);

impl ConnId for XId {
    fn generate(_socket_addr: &SocketAddr) -> Self {
        XId(rand::random())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let (node, _incoming_connections, mut incoming_messages, mut disconnections, _contact) =
        Endpoint::<XId>::new(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 5555)),
            &[],
            Config {
                idle_timeout: Duration::from_secs(60 * 5).into(),
                ..Default::default()
            },
        )
        .await?;

    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()?;

    log::info!("Server started at {}", node.public_addr());

    let (incoming_tx, mut incoming_rx): (
        Sender<(SocketAddr, Bytes)>,
        Receiver<(SocketAddr, Bytes)>,
    ) = mpsc::channel(32);

    let (outgoing_tx, mut outgoing_rx) = mpsc::channel(32);

    let (error_tx, mut error_rx) = mpsc::channel(32);

    tokio::spawn(async move {
        // Hashmap of connection id with queue name
        let mut connection_map = HashMap::<SocketAddr, String>::new();

        // Main queue to work as a buffer
        let mut main_queue = HashMap::<String, VecDeque<String>>::new();

        // Available consumers on a queue
        let mut available_queue = HashMap::<String, VecDeque<SocketAddr>>::new();

        loop {
            select! {
                Some((addr, msg)) = incoming_rx.recv() => {
                    let message_str = from_utf8(&msg).unwrap().to_string();

                    if connection_map.contains_key(&addr) == false {
                        connection_map.insert(addr.clone(), message_str.clone());

                        if main_queue.contains_key(&message_str) == false {
                            let temp1: VecDeque<String> = VecDeque::new();
                            main_queue.insert(message_str.clone(), temp1);

                            let temp: VecDeque<SocketAddr> = VecDeque::new();
                            available_queue.insert(message_str.clone(), temp);
                        }
                    } else {
                        let queue_name = connection_map.get(&addr);
                        match queue_name {
                            Some(q) => {
                                if main_queue.contains_key(q) {
                                    if message_str.starts_with(ACK) {
                                        let msg = main_queue.get_mut(q).unwrap().pop_front();
                                        if msg.is_some() {
                                            let res = outgoing_tx.send((addr, msg.unwrap())).await;
                                            if res.is_err() {
                                                log::error!("Error sending message to cosumer.");
                                            }
                                        } else {
                                            available_queue.get_mut(q).unwrap().push_back(addr);
                                        }
                                    } else {
                                        loop {
                                            match available_queue.get_mut(q).unwrap().pop_front() {
                                                Some(i) => {
                                                    if connection_map.contains_key(&i) {
                                                        let res = outgoing_tx.send((i, message_str.clone())).await;
                                                        if res.is_err() {
                                                            println!("Error occured");
                                                        }
                                                        break;
                                                    }
                                                }
                                                None => {
                                                    main_queue
                                                        .get_mut(q)
                                                        .unwrap()
                                                        .push_back(message_str.clone());
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    log::error!("Queue not found: {}", q);
                                }
                            }
                            None => {
                                log::error!("Connection {} not connected to any queue", addr);
                            }
                        }
                    }
                }
                Some(addr) = error_rx.recv() => {
                    connection_map.remove(&addr);
                }
            }
        }
    });

    loop {
        select! {
            Some((addr, msg)) = incoming_messages.next() => {
                incoming_tx.send((addr, msg)).await?;
                log::debug!("Message received from {}", addr);
            }
            Some((addr, msg)) = outgoing_rx.recv() => {
                let msg_bytes = Bytes::from(msg.clone());

                match node.connect_to(&addr).await {
                    Ok(conn) => {
                        match conn.send_with(msg_bytes.clone(), 0, None).await {
                            Ok(_) => {
                                log::debug!("Message sent to {}", addr);
                            }
                            Err(_) => {
                                log::error!("Message sending failed to {}", addr);

                                incoming_tx.send((addr, msg_bytes)).await?;
                                log::debug!("Reusing same queue for failed message");
                            }
                        };
                    }
                    Err(_) => {
                        log::error!("Connection failed to {}", addr);

                        incoming_tx.send((addr, msg_bytes)).await?;
                        error_tx.send(addr).await?;
                        log::debug!("Reusing same queue for failed message");
                    }
                }

            }
            Some(addr) = disconnections.next() => {
                log::debug!("Disconnected {}", addr);
                error_tx.send(addr).await?;
            }
        }
    }
}
