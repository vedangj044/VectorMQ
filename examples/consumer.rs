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
    let (node, _incoming_conns, mut incoming_messages, _disconnections, _contact) =
        Endpoint::<XId>::new(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            &[],
            Config {
                idle_timeout: Duration::from_secs(60 * 60).into(),
                ..Default::default()
            },
        )
        .await?;

    println!("Consumes {}", node.public_addr());

    let conn = node
        .connect_to(&SocketAddr::from((Ipv4Addr::LOCALHOST, 5555)))
        .await?;

    conn.send(Bytes::from("vedang__con")).await.unwrap();

    while let Some((_addr, message)) = incoming_messages.next().await {
        println!("Received {}", from_utf8(&message).unwrap().to_string());
    }

    Ok(())
}
