use anyhow::Result;
use bytes::Bytes;
use qp2p::{Config, ConnId, Endpoint};
use std::{
    collections::VecDeque,
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::select;

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

    println!("Started {:?}", node.public_addr());

    let peer: SocketAddr = SocketAddr::from((Ipv4Addr::LOCALHOST, 5555));
    node.connect_to(&peer).await?;

    println!("Connected to consumer");

    let mut buf: VecDeque<String> = VecDeque::new();

    loop {
        select! {
            _result = tokio::time::sleep(Duration::from_millis(100)) => {
                node.send_message({
                    if !buf.is_empty() {
                        let mesg = buf.pop_front();
                        Bytes::from(mesg.unwrap())

                    }
                    else {
                        Bytes::default()
                    }
                }, &peer, 0).await?;
                // tokio::time::sleep(Duration::from_millis(100)).await;
            }
            result = incoming_messages.next() => {
                buf.push_back(format!("{:?}", result.unwrap().1));
            }
        }
    }

    // while let Some((_socker_addr, bytes)) = incoming_messages.next().await {
    //     buf.push_back(format!("{:?}", bytes));
    // };
    // Ok(())
}
