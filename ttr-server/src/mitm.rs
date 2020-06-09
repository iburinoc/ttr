use std::error::Error as StdError;

use futures::{
    future::FutureExt,
    pin_mut, select,
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
};
use log::*;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

use ttr_net::{
    connection::Connection,
    mdns::{self, Server},
};
use ttr_protocol::{ClientMessage, Message, Response, ServerMessage};

use super::util;

#[derive(Clone)]
pub struct Mitm {
    target: Server,
    registered_as: Server,
    log_path: Option<String>,
}

#[derive(Error, Debug)]
enum MitmError {
    #[error("Connection closed unexpectedly")]
    ConnectionClosed,
}

impl Mitm {
    pub fn new(target: Server, registered_as: Server, log_path: Option<String>) -> Mitm {
        Mitm {
            target,
            registered_as,
            log_path,
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
                           util::log_packet(self.log_path.clone(), m.clone(), "c2s", i);
                           debug!("Received {:?} from client ({})", m, i);
                           debug!("Sending  {:?} to server ({})", m, i);
                           sender.send(m).await?
                       },
                       None => return Err(MitmError::ConnectionClosed.into()),
                   }
               }
               m = server_recv => {
                   match m {
                       Some(m) => {
                           util::log_packet(self.log_path.clone(), m.clone(), "s2c", i);
                           debug!("Received {:?} from server ({})", m, i);
                           let m = self.filter_server_to_client(m);
                           debug!("Sending  {:?} to client ({})", m, i);
                           output.send(m).await?
                       },
                       None => return Err(MitmError::ConnectionClosed.into()),
                   }
               }
            }

            i += 1;
        }
    }

    fn filter_server_to_client(&self, msg: ServerMessage) -> ServerMessage {
        use Message::*;
        use Response::*;
        match msg {
            Action(a) => {
                let a = match a {
                    ConnectedPlayers(mut c) => {
                        c.players
                            .iter_mut()
                            .filter(|p| p.uuid == self.target.uuid.to_hyphenated_ref().to_string())
                            .for_each(|p| {
                                p.uuid = self.registered_as.uuid.to_hyphenated_ref().to_string();
                                p.name = self.registered_as.name.clone();
                            });
                        ConnectedPlayers(c)
                    }
                    a => a,
                };
                Action(a)
            }
            Connect(mut c) => {
                c.peerId = self.registered_as.peer_id;
                c.name = self.registered_as.name.clone();
                Connect(c)
            }
            msg => msg,
        }
    }
}

pub async fn run(args: super::MitmArgs) -> anyhow::Result<()> {
    let mut server = TcpListener::bind((args.host.as_str(), args.port)).await?;
    let addr = server.local_addr()?;
    info!("Tcp server bound to {:?}", addr);

    let target = util::find_server().await?;
    info!("Found mitm target {:?}", target);

    let mut fake_server = target.clone();
    fake_server.address = addr;
    fake_server.name += "mitm";
    fake_server.peer_id = args
        .player_id
        .peer_id
        .unwrap_or_else(|| rand::random::<u64>());
    fake_server.uuid = args.player_id.uuid.unwrap_or_else(|| Uuid::new_v4());

    let _registration = mdns::register(&fake_server).await?;
    info!("Registered as {:?}", fake_server);

    loop {
        let (stream, addr) = server.accept().await?;
        info!("New connection from {:?}", addr);
        let mitm = Mitm::new(target.clone(), fake_server.clone(), args.log_path.clone());
        tokio::spawn(async move {
            match handle_stream(stream, mitm).await {
                Ok(()) => info!("{:?} finished", addr),
                Err(e) => error!("Error occurred for {:?}: {:?}", addr, e),
            }
        });
    }
}

async fn handle_stream(stream: TcpStream, mitm: Mitm) -> anyhow::Result<()> {
    let (_connection, receiver, sender) = Connection::from_stream(stream);
    mitm.run(receiver, sender).await
}
