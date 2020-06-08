use std::{error::Error as StdError, time::Duration};

use anyhow::Context;
use futures::{
    future::FutureExt,
    pin_mut, select,
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
};
use log::*;
use thiserror::Error;

use ttr_net::mdns::Server;
use ttr_protocol::{Action, ClientMessage, Message, ServerMessage};

#[derive(Clone)]
pub struct Mitm {
    target: Server,
    registered_as: Server,
    unrecognized_path: Option<String>,
}

#[derive(Error, Debug)]
enum MitmError {
    #[error("No ttr server to target found")]
    NoServerFound,
    #[error("Connection closed unexpectedly")]
    ConnectionClosed,
}

impl Mitm {
    pub fn new(target: Server, registered_as: Server, unrecognized_path: Option<String>) -> Mitm {
        Mitm {
            target,
            registered_as,
            unrecognized_path,
        }
    }

    pub async fn run<T>(
        self,
        input: impl Stream<Item = ClientMessage>,
        output: T,
    ) -> anyhow::Result<()>
    where
        T: Sink<ServerMessage>,
        T::Error: StdError + Send + Sync + 'static,
    {
        let (_connection, receiver, sender) = ttr_net::connect(self.target.address).await?;

        let mut i = 0;

        pin_mut!(input, output, receiver, sender);
        loop {
            let client_recv = input.next().fuse();
            let server_recv = receiver.next().fuse();
            pin_mut!(client_recv, server_recv);

            select! {
               m = client_recv => {
                   match m {
                       Some(m) => {
                           self.log_unrecog(m.clone(), "c2s", i);
                           debug!("Received {:?} from client", m);
                           let m = self.filter_client_to_server(m);
                           debug!("Sending  {:?} to server", m);
                           sender.send(m).await?
                       },
                       None => return Err(MitmError::ConnectionClosed.into()),
                   }
               }
               m = server_recv => {
                   match m {
                       Some(m) => {
                           self.log_unrecog(m.clone(), "s2c", i);
                           debug!("Received {:?} from server", m);
                           let m = self.filter_server_to_client(m);
                           debug!("Sending  {:?} to client", m);
                           output.send(m).await?
                       },
                       None => return Err(MitmError::ConnectionClosed.into()),
                   }
               }
            }

            i += 1;
        }
    }

    fn log_unrecog<A: Action>(&self, m: Message<A>, typ: &'static str, idx: i32) {
        let path = match &self.unrecognized_path {
            Some(p) => p.clone(),
            None => return,
        };
        match m {
            Message::Unrecognized { kind, data } => {
                tokio::spawn(async move {
                    let path = format!("{}{}_{}_k{}", path, idx, typ, kind);
                    tokio::fs::write(path, data).await.unwrap();
                });
            }
            _ => {}
        }
    }

    fn filter_server_to_client(&self, msg: ServerMessage) -> ServerMessage {
        use Message::*;
        match msg {
            Connect(mut c) => {
                c.peerId = self.registered_as.peer_id;
                c.name = self.registered_as.name.clone();
                Connect(c)
            }
            msg => msg,
        }
    }

    fn filter_client_to_server(&self, msg: ClientMessage) -> ClientMessage {
        use Message::*;
        match msg {
            msg => msg,
        }
    }
}

pub async fn find_target() -> anyhow::Result<Server> {
    let target = ttr_net::browse(Duration::from_secs(3))
        .await
        .context("Error looking for server")?
        .ok_or(MitmError::NoServerFound)?;

    Ok(target)
}
