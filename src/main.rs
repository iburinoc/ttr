use std::{error::Error as StdError, net::SocketAddr, time::Duration};

use async_dnssd::{RegisterData, Registration, StreamTimeoutExt, TxtRecord};
use futures::prelude::*;
use log::*;
use structopt::StructOpt;
use thiserror::Error;
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
#[structopt(name = "ttr", about = "Ticket to ride app man in the middle server")]
struct Opt {
    /// Hostname to bind to
    #[structopt(short = "H", long, default_value = "127.0.0.1")]
    host: String,
}

const MDNS_TYPE: &'static str = "_t2rdaysofwonder._tcp";

async fn register(port: u16) -> anyhow::Result<Registration> {
    let uuid = Uuid::new_v4().to_hyphenated().to_string();
    let mut record = TxtRecord::new();
    record
        .set_value("gameStatus".as_bytes(), "1".as_bytes())
        .unwrap();
    record
        .set_value("platform".as_bytes(), "generic".as_bytes())
        .unwrap();
    record
        .set_value("uuid".as_bytes(), uuid.as_bytes())
        .unwrap();
    record
        .set_value("version".as_bytes(), "2.7.6".as_bytes())
        .unwrap();
    record
        .set_value("_d".as_bytes(), "sean".as_bytes())
        .unwrap();
    let (registration, result) = async_dnssd::register_extended(
        MDNS_TYPE,
        port,
        RegisterData {
            txt: record.data(),
            ..Default::default()
        },
    )?
    .await?;
    debug!("Registered: {:?}", result);
    Ok(registration)
}

async fn find_server() -> anyhow::Result<SocketAddr> {
    let browse = async_dnssd::browse(MDNS_TYPE)?.timeout(Duration::from_secs(3))?;
    let val = browse
        .map(|service| async move {
            debug!("Found service: {:?}", service);
            service
        })
        .buffer_unordered(16)
        .next()
        .await;
    debug!("val: {:?}", val);
    Err(FindError::NoServerFound.into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    env_logger::builder()
        .filter(Some("ttr"), LevelFilter::Debug)
        .init();

    let args = Opt::from_args();

    let mut server = TcpListener::bind((args.host.as_str(), 0)).await?;
    let addr = server.local_addr()?;
    debug!("Tcp server bound to {:?}", addr);
    let port = addr.port();

    let remote_host = find_server().await?;

    let _registration = register(port).await?;

    loop {
        let (stream, addr) = server.accept().await?;
        debug!("New connection from {:?}", addr);
    }
}

#[derive(Error, Debug)]
enum FindError {
    #[error("No ttr server found")]
    NoServerFound,
}
