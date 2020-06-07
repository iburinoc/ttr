use std::{error::Error as StdError, time::Duration};

use anyhow::Context;
use futures::{
    future::FutureExt,
    pin_mut, select,
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
};
use thiserror::Error;

use ttr_net::mdns::Server;
use ttr_protocol::Message;

pub struct Mitm {
    target: Server,
}

#[derive(Error, Debug)]
enum MitmError {
    #[error("No ttr server to target found")]
    NoServerFound,
    #[error("Connection closed unexpectedly")]
    ConnectionClosed,
}

impl Mitm {
    pub async fn new() -> anyhow::Result<Mitm> {
        let target = ttr_net::browse(Duration::from_secs(3))
            .await
            .context("Error looking for server")?
            .ok_or(MitmError::NoServerFound)?;

        Ok(Mitm { target })
    }

    pub fn target(&self) -> &Server {
        &self.target
    }

    pub async fn run<T>(
        &self,
        registered_as: Server,
        input: impl Stream<Item = Message>,
        output: T,
    ) -> anyhow::Result<()>
    where
        T: Sink<Message>,
        T::Error: StdError + Send + Sync + 'static,
    {
        let (_connection, receiver, sender) = ttr_net::connect(self.target.address).await?;

        pin_mut!(input, output, receiver, sender);
        loop {
            let client_recv = input.next().fuse();
            let server_recv = receiver.next().fuse();
            pin_mut!(client_recv, server_recv);

            select! {
                m = client_recv => {
                    match m {
                        Some(m) => sender.send(m).await?,
                        None => return Err(MitmError::ConnectionClosed.into()),
                    }
                }
                m = server_recv => {
                    match m {
                        Some(m) => output.send(m).await?,
                        None => return Err(MitmError::ConnectionClosed.into()),
                    }
                }
            }
        }
    }
}
