use anyhow::Result;
// use bytes::Bytes;
use qp2p::{Config, ConnId, Endpoint};
use std::{
    collections::{HashMap, VecDeque},
    net::{Ipv4Addr, SocketAddr},
    ops::Mul,
    str::from_utf8,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;
use tokio::{runtime::Handle, select};

#[derive(Default, Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
struct XId(pub [u8; 32]);

impl ConnId for XId {
    fn generate(_socket_addr: &SocketAddr) -> Self {
        XId(rand::random())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // let queue_mapping: Arc<Mutex<HashMap<String, VecDeque<String>>>> =
    // Arc::new(Mutex::new(HashMap::new()));
    let connections: Arc<Mutex<HashMap<SocketAddr, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut queue_test: Arc<Mutex<HashMap<String, mpsc::Sender<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let (node, _incoming_conns, mut incoming_messages, _disconnections, _contact) =
        Endpoint::<XId>::new(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 5555)),
            &[],
            Config {
                idle_timeout: Duration::from_secs(60 * 60).into(),
                ..Default::default()
            },
        )
        .await?;

    loop {
        select! {
            queue_id = incoming_messages.next() => {
                match queue_id {
                    Some((socker_addr, queue_name_bytes)) => {
                        let q = from_utf8(&queue_name_bytes).unwrap().to_string();
                        println!("{}", q);

                        let mut dd = connections.lock().unwrap();
                        if dd.contains_key(&socker_addr) {
                            let q1 = dd.get(&socker_addr).unwrap();
                            // let mut db = queue_mapping.lock().unwrap();
                            // db.get_mut(q1).unwrap().push_back(q);

                            let mut db = queue_test.lock().unwrap();
                            let fuu = db.get_mut(q1).unwrap().send(q.clone()).await;
                            match fuu {
                                Err(e) => {
                                    println!("{}", e);
                                }
                                _ => {

                                }
                            };
                            // println!("Message {}, {}", q, fuu.is_ok());

                        } else {
                            let v: Vec<&str> = q.split("__").collect();

                            let queue_name = v.get(0).unwrap().to_owned();
                            let con = v.get(1).unwrap().to_string();

                            let (tx, mut rx) = mpsc::channel(32);

                            let mut db = queue_test.lock().unwrap();
                            db.insert(queue_name.to_string(), tx);


                            let consumer;
                            if con == "con".to_string() {
                                consumer = true;

                                tokio::spawn(async move {
                                    // println!("Bhai");
                                    while let Some(message) = rx.recv().await {
                                        println!("hey, {}", message);
                                    }
                                });

                            } else {
                                consumer = false;
                            }
                            println!("queue_name: {}, consumer : {:?}", queue_name, consumer);

                            // let ve = VecDeque::new();

                            // let mut db = queue_mapping.lock().unwrap();
                            // db.insert(queue_name.to_string(), ve);
                            // hashmap.insert(queue_name.to_string(), ve);

                            // let mut cn = connections.lock().unwrap();
                            dd.insert(socker_addr, queue_name.to_string());
                        }
                    }
                    None => {
                        println!("Invalid queue name");
                    }
                }
            }
        }

        // Creating a connection
        // let queue_id = incoming_messages.next().await;
    }
}
