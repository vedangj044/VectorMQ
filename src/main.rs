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
// use tokio::{runtime::Handle, select};
use tokio::sync::mpsc;

#[derive(Default, Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
struct XId(pub [u8; 32]);

impl ConnId for XId {
    fn generate(_socket_addr: &SocketAddr) -> Self {
        XId(rand::random())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let queue_mapping: Arc<Mutex<HashMap<String, VecDeque<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let connections: Arc<Mutex<HashMap<SocketAddr, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let (_node, _incoming_conns, mut incoming_messages, _disconnections, _contact) =
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
        // Creating a connection
        let queue_id = incoming_messages.next().await;
        match queue_id {
            Some((socker_addr, queue_name_bytes)) => {
                let q = from_utf8(&queue_name_bytes).unwrap().to_string();

                let dd = connections.lock().unwrap();
                if dd.contains_key(&socker_addr) {
                    let q1 = dd.get(&socker_addr).unwrap();
                    let mut db = queue_mapping.lock().unwrap();
                    db.get_mut(q1).unwrap().push_back(q);
                } else {
                    let v: Vec<&str> = q.split("__").collect();

                    let queue_name = v.get(0).unwrap().to_owned();
                    let con = v.get(1).unwrap().to_string();

                    let consumer;
                    if con == "con".to_string() {
                        consumer = true;
                    } else {
                        consumer = false;
                    }
                    println!("queue_name: {}, consumer : {:?}", queue_name, consumer);

                    let ve = VecDeque::new();

                    let mut db = queue_mapping.lock().unwrap();
                    db.insert(queue_name.to_string(), ve);

                    // hashmap.insert(queue_name.to_string(), ve);

                    let mut cn = connections.lock().unwrap();
                    cn.insert(socker_addr, queue_name.to_string());
                }
            }
            None => {
                println!("Invalid queue name");
            }
        }
    }
}
