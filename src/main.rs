use std::{env, time::Duration};

use async_dnssd::{RegisterData, ResolvedHostFlags, StreamTimeoutExt, TxtRecord};
use futures::prelude::*;
use tokio::spawn;
use uuid::Uuid;

#[tokio::main(basic_scheduler)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let search_timeout = Duration::from_secs(10);
    let resolve_timeout = Duration::from_secs(3);
    let address_timeout = Duration::from_secs(3);

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
    let _register = async_dnssd::register_extended(
        "_t2rdaysofwonder._tcp",
        64444,
        RegisterData {
            txt: record.data(),
            ..Default::default()
        },
    )?
    .await?;

    loop {
        tokio::time::delay_for(Duration::from_secs(10)).await;
    }
}
