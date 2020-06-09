use futures::{pin_mut, sink::SinkExt, stream::StreamExt};
use log::*;
use uuid::Uuid;

use ttr_net::connection;
use ttr_protocol::{
    protos::{Connect, Hello},
    Message, Query,
};

use super::util;

pub async fn run(args: super::DummyArgs) -> anyhow::Result<()> {
    let server = util::find_server().await?;
    info!("Found server {:?}", server);

    let (connection, receiver, mut sender) = connection::connect(server.address).await?;

    let peer_id = args
        .player_id
        .peer_id
        .unwrap_or_else(|| rand::random::<u64>());
    let uuid = args.player_id.uuid.unwrap_or_else(|| Uuid::new_v4());

    info!("Peer id: {}", peer_id);
    info!("Uuid:    {}", uuid.to_hyphenated_ref());

    pin_mut!(receiver);

    sender
        .send(Message::Connect(Connect {
            name: args.name.clone(),
            peerId: peer_id,
            ctx: String::from("myContext"),
            ..Default::default()
        }))
        .await?;

    let server_connect = receiver.next().await.ok_or(DummyError::UnexpectedClose)?;
    info!("Server connect: {:?}", server_connect);

    sender
        .send(Message::Action(Query::Hello(Hello {
            name: args.name.clone(),
            uuid: uuid.to_hyphenated_ref().to_string(),
            colorId: -1,
            protocolVersion: 1,
            ..Default::default()
        })))
        .await?;

    let mut i = 3;
    receiver
        .for_each(|msg| {
            info!("Received #{} {:?}", i, msg);
            util::log_packet(args.log_path.clone(), msg, "s2c", i);
            i += 1;
            async { () }
        })
        .await;

    connection.close().await?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum DummyError {
    #[error("Stream closed unexpectedly")]
    UnexpectedClose,
}
