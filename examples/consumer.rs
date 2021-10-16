use anyhow::{anyhow, Result};
use bytes::Bytes;
use qp2p::{Config, ConnId, Connection, Endpoint};
use std::{net::*, str::from_utf8, time::Duration};

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

    println!("Consumer started on {}", node.public_addr());

    let conn = node
        .connect_to(&SocketAddr::from((Ipv4Addr::LOCALHOST, 5555)))
        .await?;

    println!("Connected to server");

    conn.send(Bytes::from("queue1")).await.unwrap();
    println!("Connected to queue - queue1");

    ack(&conn).await?;

    while let Some((_addr, message)) = incoming_messages.next().await {
        println!("Received {}", from_utf8(&message).unwrap().to_string());
        ack(&conn).await?;
    }

    Ok(())
}

async fn ack(conn: &Connection<XId>) -> Result<()> {
    conn.send(Bytes::from("###ack###"))
        .await
        .map_err(|e| anyhow!("Error ack {}", e))
}
