use anyhow::Result;
use bytes::Bytes;
use qp2p::{Config, ConnId, Endpoint};
use std::{
    collections::{HashMap, VecDeque},
    net::{Ipv4Addr, SocketAddr},
    ptr::NonNull,
    str::from_utf8,
    time::Duration,
};
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
    let mut hashmap: HashMap<String, VecDeque<String>> = HashMap::new();
    let vec: VecDeque<String> = VecDeque::new();

    let mut i = 1;
    loop {
        if i == 0 {
            break;
        }
        i -= 1;

        tokio::spawn(async move {
            // Creating a connection
            let (node, incoming_conns, mut incoming_messages, _disconnections, _contact) =
                Endpoint::<XId>::new(
                    SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
                    &[],
                    Config {
                        idle_timeout: Duration::from_secs(60 * 60).into(),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();

            println!("Connectes {}", node.public_addr());

            let conn = node
                .connect_to(&SocketAddr::from((Ipv4Addr::LOCALHOST, 5555)))
                .await
                .unwrap();

            conn.send(Bytes::from("vedang__pub")).await.unwrap();

            conn.send(Bytes::from("123")).await.unwrap();
            conn.send(Bytes::from("234")).await.unwrap();
            conn.send(Bytes::from("345")).await.unwrap();
            conn.send(Bytes::from("456")).await.unwrap();
            conn.send(Bytes::from("567")).await.unwrap();
            conn.send(Bytes::from("print")).await.unwrap();
        })
        .await;
    }
    Ok(())
}
