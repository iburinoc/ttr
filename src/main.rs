use std::{collections::HashMap, error::Error as StdError, net::SocketAddr, time::Duration};

use async_dnssd::{
    BrowsedFlags, RegisterData, Registration, ResolvedHostFlags, ScopedSocketAddr,
    StreamTimeoutExt, TxtRecord,
};
use futures::{pin_mut, prelude::*};
use log::*;
use structopt::StructOpt;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

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

const MDNS_TYPE: &'static str = "_t2rdaysofwonder._tcp";
static TIMEOUT: Duration = Duration::from_secs(1);

async fn register(port: u16, kvs: HashMap<String, String>) -> anyhow::Result<Registration> {
    let record = kvs
        .iter()
        .fold(TxtRecord::new(), |mut record, (key, value)| {
            let value = if key == "_d" {
                format!("{}", value)
            } else {
                value.clone()
            };
            record.set_value(key.as_bytes(), value.as_bytes()).unwrap();
            record
        });
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

async fn find_server() -> anyhow::Result<(SocketAddr, HashMap<String, String>)> {
    let browse = async_dnssd::browse(MDNS_TYPE)?.timeout(Duration::from_secs(3))?;
    let stream = browse
        .map_err(anyhow::Error::new)
        .try_filter_map(|service| async move {
            let result: anyhow::Result<_> = async {
                let added = service.flags.contains(BrowsedFlags::ADD);
                if !added {
                    return Ok(None);
                }
                debug!("Found service: {:?}", service);
                let resolve = service
                    .resolve()?
                    .timeout(TIMEOUT)?
                    .map_err(anyhow::Error::new);
                let name = service.service_name.clone();
                let res = resolve.try_filter_map(move |r| {
                    let name = name.clone();
                    async move {
                        let txt = TxtRecord::parse(&r.txt)
                            .map(|rdata| {
                                rdata
                                    .iter()
                                    .filter_map(|(key, value)| {
                                        let from_utf8 =
                                            |x| String::from(String::from_utf8_lossy(x));
                                        value.map(|v| (from_utf8(key), from_utf8(v)))
                                    })
                                    .collect::<HashMap<_, _>>()
                            })
                            .ok_or(FindError::DataParseError)?;
                        let addr = Box::pin(
                            r.resolve_socket_address()?
                                .timeout(Duration::from_secs(1))?
                                .map_err(anyhow::Error::new)
                                .filter_map(|x| async {
                                    match x {
                                        Ok(x) => Some(x),
                                        Err(e) => {
                                            error!("Resolution error: {:?}", e);
                                            None
                                        }
                                    }
                                })
                                .filter_map(|result| async {
                                    match result.address {
                                        ScopedSocketAddr::V4 { .. } => Some(result),
                                        ScopedSocketAddr::V6 { .. } => None,
                                    }
                                })
                                .filter_map(|result| async {
                                    if result.flags.intersects(ResolvedHostFlags::ADD) {
                                        Some(result.address)
                                    } else {
                                        None
                                    }
                                }),
                        )
                        .next()
                        .await
                        .ok_or(FindError::NoResolutionFound)?;
                        debug!(
                            "Resolved {:?} on {:?}: {:?}:{}, txt: {:?}",
                            name, r.interface, addr, r.port, txt
                        );
                        Ok(Some((addr.into(), txt)))
                    }
                });
                Ok(Some(res))
            }
            .await;
            result
        })
        .try_flatten()
        .filter_map(|x| async {
            match x {
                Ok(x) => Some(x),
                Err(e) => {
                    error!("Resolution error: {:?}", e);
                    None
                }
            }
        });
    pin_mut!(stream);
    let val = stream.next().await;
    debug!("val: {:?}", val);
    match val {
        Some(x) => Ok(x),
        None => Err(FindError::NoServerFound.into()),
    }
}

async fn transfer(r: impl AsyncRead, w: impl AsyncWrite) {
    let mut buf = vec![0u8; 65536];
    pin_mut!(r);
    pin_mut!(w);
    loop {
        let res = r.read(&mut buf).await;
        let len = match res {
            Ok(l) => l,
            Err(e) => {
                error!("Read error: {:?}", e);
                return;
            }
        };
        let res = w.write_all(&buf[0..len]).await;
        match res {
            Ok(()) => (),
            Err(e) => {
                error!("Write error: {:?}", e);
                return;
            }
        }
    }
}

async fn mitm_connection(host1: TcpStream, host2: TcpStream) {
    let (read1, write1) = host1.into_split();
    let (read2, write2) = host2.into_split();
    future::join(transfer(read1, write2), transfer(read2, write1)).await;
}

async fn handle_stream(stream: TcpStream, remote_host: SocketAddr) -> anyhow::Result<()> {
    let other = TcpStream::connect(remote_host).await?;
    debug!("Connected to {:?}", remote_host);
    mitm_connection(stream, other).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    env_logger::builder()
        .filter(Some("ttr"), LevelFilter::Debug)
        .init();

    let args = Opt::from_args();

    let mut server = TcpListener::bind((args.host.as_str(), args.port)).await?;
    let addr = server.local_addr()?;
    debug!("Tcp server bound to {:?}", addr);
    let port = addr.port();

    let (remote_host, kvs) = find_server().await?;

    debug!("Found game host {:?} with kvs {:?}", remote_host, kvs);

    let _registration = register(port, kvs).await?;

    loop {
        let (stream, addr) = server.accept().await?;
        debug!("New connection from {:?}", addr);
        tokio::spawn(async move {
            match handle_stream(stream, remote_host).await {
                Ok(()) => debug!("{:?} finished", addr),
                Err(e) => error!("Error occurred for {:?}: {:?}", addr, e),
            }
        });
    }
}

#[derive(Error, Debug)]
enum FindError {
    #[error("No ttr server found")]
    NoServerFound,
    #[error("Failed to parse data")]
    DataParseError,
    #[error("No resolutions found")]
    NoResolutionFound,
}
