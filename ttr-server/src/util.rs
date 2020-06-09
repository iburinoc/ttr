use std::time::Duration;

use anyhow::Context;
use protobuf::Message as _;
use thiserror::Error;

use ttr_net::mdns::Server;
use ttr_protocol::{Action, Message};

pub fn log_packet<A: Action>(path: Option<String>, m: Message<A>, typ: &'static str, idx: i32) {
    tokio::spawn(async move {
        let path = match path {
            Some(p) => p.clone(),
            None => return,
        };
        let (kind, data) = match m {
            Message::Action(a) => (1, Action::to_proto(a).write_to_bytes().unwrap()),
            Message::Heartbeat(h) => (2, h.write_to_bytes().unwrap()),
            Message::Connect(c) => (3, c.write_to_bytes().unwrap()),
        };
        let path = format!("{}{}_{}_k{}", path, idx, typ, kind);
        tokio::fs::write(path, data).await.unwrap();
    });
}

pub async fn find_server() -> anyhow::Result<Server> {
    let server = ttr_net::browse(Duration::from_secs(3))
        .await
        .context("Error looking for server")?
        .ok_or(FindError::NoServerFound)?;

    Ok(server)
}

#[derive(Error, Debug)]
enum FindError {
    #[error("No ttr server found")]
    NoServerFound,
}
