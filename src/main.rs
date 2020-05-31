use std::{env, error::Error as StdError, time::Duration};

use async_dnssd::{RegisterData, Registration, ResolvedHostFlags, StreamTimeoutExt, TxtRecord};
use futures::prelude::*;
use log::*;
use tokio::spawn;
use uuid::Uuid;

async fn register(port: u16) -> Result<Registration, Box<dyn StdError>> {
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
        "_t2rdaysofwonder._tcp",
        64444,
        RegisterData {
            txt: record.data(),
            ..Default::default()
        },
    )?
    .await?;
    debug!("Registered: {:?}", result);
    Ok(registration)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    env_logger::init();

    let _registration = register(64444).await?;

    loop {
        tokio::time::delay_for(Duration::from_secs(10)).await;
    }
}
