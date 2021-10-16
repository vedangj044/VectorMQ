use anyhow::Result;
use bytes::Bytes;
use qp2p::{Config, ConnId, Endpoint};
use std::{net::*, time::Duration};

#[derive(Default, Ord, PartialEq, PartialOrd, Eq, Clone, Copy)]
struct XId(pub [u8; 32]);

impl ConnId for XId {
    fn generate(_socket_addr: &SocketAddr) -> Self {
        XId(rand::random())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let (node, _incoming_conns, mut _incoming_messages, _disconnections, _contact) =
        Endpoint::<XId>::new(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            &[],
            Config {
                idle_timeout: Duration::from_secs(60 * 60).into(),
                ..Default::default()
            },
        )
        .await?;

    println!("Prodcuer started on {}", node.public_addr());

    let conn = node
        .connect_to(&SocketAddr::from((Ipv4Addr::LOCALHOST, 5555)))
        .await?;

    println!("Connected to server");

    conn.send(Bytes::from("queue1")).await.unwrap();
    println!("Connected to queue - queue1");

    for i in 1..50000 {
        conn.send(Bytes::from(format!("This is a new message, value {}", i)))
            .await?;
    }

    Ok(())
}
