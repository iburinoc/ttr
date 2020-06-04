use std::{net::SocketAddr, time::Duration};

use log::*;
use uuid::Uuid;

pub use async_dnssd::{Error as DnssdError, Registration};
use async_dnssd::{RegisterData, TxtRecord};

const MDNS_TYPE: &'static str = "_t2rdaysofwonder._tcp";

#[derive(Debug, Clone)]
pub struct Server {
    pub address: SocketAddr,
    pub name: String,
    pub peer_id: u64,
    pub uuid: Uuid,
    pub opaque: u64, // TODO: Figure out what this means.  Seems related to map.
}

impl Server {
    fn gen_record(&self) -> TxtRecord {
        let mut record = TxtRecord::new();
        record
            .set_value("opaque".as_bytes(), self.opaque.to_string().as_bytes())
            .unwrap();
        record
            .set_value("gameStatus".as_bytes(), "1".as_bytes())
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
    let (registration, result) = async_dnssd::register_extended(
        MDNS_TYPE,
        server.address.port(),
        RegisterData {
            txt: record.data(),
            ..Default::default()
        },
    )?
    .await?;
    debug!("Registered {:?} for {:?}", result, server);
    Ok(registration)
}
