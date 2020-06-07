use std::{error::Error as StdError, net::SocketAddr};

use futures::{pin_mut, prelude::*};
use log::*;
use structopt::StructOpt;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use uuid::Uuid;

use ttr_net::{connection::Connection, mdns};

mod mitm;

use mitm::Mitm;

#[derive(Debug, StructOpt)]
#[structopt(name = "ttr", about = "Ticket to ride app man in the middle server")]
struct Opt {
    /// Hostname to bind to
    #[structopt(short = "H", long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind to
    #[structopt(short, long, default_value = "0")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    env_logger::builder()
        .filter(Some("ttr"), LevelFilter::Info)
        .init();

    let args = Opt::from_args();

    let mut server = TcpListener::bind((args.host.as_str(), args.port)).await?;
    let addr = server.local_addr()?;
    info!("Tcp server bound to {:?}", addr);
    let port = addr.port();

    let mitm = Mitm::new().await?;
    info!("Found mitm target {:?}", mitm.target());

    let mut fake_server = mitm.target().clone();
    fake_server.uuid = Uuid::new_v4();
    fake_server.address = addr;
    fake_server.name += "mitm";
    fake_server.peer_id = rand::random::<u64>();

    let _registration = mdns::register(&fake_server).await?;
    info!("Registered as {:?}", fake_server);

    loop {
        tokio::time::delay_for(std::time::Duration::from_secs(10)).await
    }

    Ok(())
}
