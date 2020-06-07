use std::{io, net::SocketAddr, time::Duration};

use futures::{
    pin_mut,
    stream::{StreamExt, TryStreamExt},
};
use log::*;
use thiserror::Error;
use uuid::Uuid;

pub use async_dnssd::{
    BrowsedFlags, Error as DnssdError, Registration, ResolvedHostFlags, StreamTimeoutExt,
};
use async_dnssd::{RegisterData, TxtRecord};

const MDNS_TYPE: &'static str = "_t2rdaysofwonder._tcp";

#[derive(Debug, Clone)]
pub struct Server {
    pub address: SocketAddr,
    pub name: String,
    pub peer_id: u64,
    pub uuid: Uuid,
    pub opaque: String, // TODO: Figure out what this means.  Seems related to map.
    pub game_status: u32, // TODO: Figure out what this means.  Maybe player count or something?
}

#[derive(Error, Debug)]
pub enum MdnsError {
    #[error(transparent)]
    AsyncDnssdError(#[from] DnssdError),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("Missing field in server TXT record: {0}")]
    MissingTxtField(&'static str),
    #[error("Invalid TXT record: {0:?}")]
    InvalidTxt(Vec<u8>),
}

impl Server {
    fn parse(service_name: &str, txt: &[u8], address: SocketAddr) -> Result<Self, MdnsError> {
        let inv_txt = || MdnsError::InvalidTxt(txt.into());
        let txt = TxtRecord::parse(txt).ok_or_else(inv_txt)?;

        let peer_id = u64::from_str_radix(service_name, 36).map_err(|_| inv_txt())?;
        let get = |name: &'static str| {
            txt.get(name.as_bytes())
                .flatten()
                .ok_or_else(|| MdnsError::MissingTxtField(name))
        };

        let get_int = |name| {
            std::str::from_utf8(get(name)?)
                .map_err(|_| inv_txt())?
                .parse()
                .map_err(|_| inv_txt())
        };
        let get_str = |name| String::from_utf8(get(name)?.into()).map_err(|_| inv_txt());

        let opaque = get_str("opaque")?;
        let game_status = get_int("gameStatus")?;
        let name = get_str("_d")?;
        let uuid = std::str::from_utf8(get("uuid")?)
            .map_err(|_| inv_txt())?
            .parse()
            .map_err(|_| inv_txt())?;

        Ok(Server {
            address,
            name,
            peer_id,
            uuid,
            opaque,
            game_status,
        })
    }

    fn gen_record(&self) -> TxtRecord {
        let mut record = TxtRecord::new();
        record
            .set_value("opaque".as_bytes(), self.opaque.as_bytes())
            .unwrap();
        record
            .set_value(
                "gameStatus".as_bytes(),
                self.game_status.to_string().as_bytes(),
            )
            .unwrap();
        record
            .set_value(
                "uuid".as_bytes(),
                self.uuid.to_hyphenated_ref().to_string().as_bytes(),
            )
            .unwrap();
        record
            .set_value("version".as_bytes(), "2.7.6".as_bytes())
            .unwrap();
        record
            .set_value("_d".as_bytes(), self.name.as_bytes())
            .unwrap();
        record
            .set_value("platform".as_bytes(), "generic".as_bytes())
            .unwrap();

        record
    }
}

pub async fn register(server: &Server) -> Result<Registration, DnssdError> {
    let record = server.gen_record();
    let name = radix_fmt::radix(server.peer_id, 36).to_string();
    let (registration, result) = async_dnssd::register_extended(
        MDNS_TYPE,
        server.address.port(),
        RegisterData {
            txt: record.data(),
            name: Some(name.as_str()),
            ..Default::default()
        },
    )?
    .await?;
    debug!("Registered {:?} for {:?}", result, server);
    Ok(registration)
}

pub async fn browse(timeout: impl Into<Option<Duration>>) -> Result<Option<Server>, MdnsError> {
    let timeout = timeout
        .into()
        .unwrap_or(Duration::from_secs(3600 * 24 * 365 * 20));
    let browse = async_dnssd::browse(MDNS_TYPE)?.timeout(timeout)?;
    let stream = browse
        .map_err(MdnsError::IoError)
        .try_filter_map(|service| async move {
            let added = service.flags.contains(BrowsedFlags::ADD);
            if !added {
                return Ok(None);
            }
            let resolve = service.resolve()?.timeout(timeout)?;
            let addr = resolve
                .try_filter_map(move |r| {
                    let service_name = service.service_name.clone();
                    async move {
                        let s = r
                            .resolve_socket_address()?
                            .timeout(timeout)?
                            .try_filter_map(|result| async {
                                if result.flags.intersects(ResolvedHostFlags::ADD) {
                                    Ok(Some(result.address))
                                } else {
                                    Ok(None)
                                }
                            })
                            .map(move |addr| match addr {
                                Ok(addr) => Server::parse(
                                    service_name.as_str(),
                                    r.txt.as_slice(),
                                    addr.into(),
                                ),
                                Err(e) => Err(e.into()),
                            });
                        Ok(Some(s))
                    }
                })
                .try_flatten();
            pin_mut!(addr);
            addr.next().await.transpose()
        });
    pin_mut!(stream);
    stream.next().await.transpose()
}
