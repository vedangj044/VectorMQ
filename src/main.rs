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
use tokio::{
    select,
    sync::mpsc::{self, Receiver, Sender},
};

#[derive(Default, Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
struct XId(pub [u8; 32]);

impl ConnId for XId {
    fn generate(_socket_addr: &SocketAddr) -> Self {
        XId(rand::random())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let (node, _incoming_connections, mut incoming_messages, _disconnections, _contact) =
        Endpoint::<XId>::new(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 5555)),
            &[],
            Config {
                idle_timeout: Duration::from_secs(3600).into(),
                ..Default::default()
            },
        )
        .await?;

    let (tx, mut rx): (Sender<(SocketAddr, Bytes)>, Receiver<(SocketAddr, Bytes)>) =
        mpsc::channel(32);

    let (tx1, mut rx1) = mpsc::channel(32);

    tokio::spawn(async move {
        let mut connection_map = HashMap::<SocketAddr, String>::new();

        let mut main_queue = HashMap::<String, VecDeque<String>>::new();
        let mut available_queue = HashMap::<String, VecDeque<SocketAddr>>::new();

        while let Some((addr, msg)) = rx.recv().await {
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
                let queue_name = connection_map.get(&addr).unwrap();
                if message_str.starts_with("###ack###") == true {
                    let msg = main_queue.get_mut(queue_name).unwrap().pop_front();
                    if msg.is_some() {
                        let res = tx1.send((addr, msg.unwrap())).await;
                        if res.is_err() {
                            println!("Error occured hihi");
                        }
                    } else {
                        available_queue.get_mut(queue_name).unwrap().push_back(addr);
                    }
                } else {
                    match available_queue.get_mut(queue_name).unwrap().pop_front() {
                        Some(i) => {
                            let res = tx1.send((i, message_str.clone())).await;
                            if res.is_err() {
                                println!("Error occured hihi");
                            }
                        }
                        None => {
                            main_queue
                                .get_mut(queue_name)
                                .unwrap()
                                .push_back(message_str.clone());
                        }
                    }
                }
            }
        }
    });

    loop {
        select! {
            Some((addr, msg)) = incoming_messages.next() => {
                tx.send((addr, msg)).await?;
            }
            Some((addr, msg)) = rx1.recv() => {
                node.connect_to(&addr).await?.send(Bytes::from(msg.clone())).await?;
            }
        }
    }
}
