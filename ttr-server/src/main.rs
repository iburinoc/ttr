use std::error::Error as StdError;

use log::*;
use structopt::StructOpt;
use tokio::net::{TcpListener, TcpStream};
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

async fn handle_stream(stream: TcpStream, mitm: Mitm) -> anyhow::Result<()> {
    let (_connection, receiver, sender) = Connection::from_stream(stream);
    mitm.run(receiver, sender).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    env_logger::builder().init();

    let args = Opt::from_args();

    let mut server = TcpListener::bind((args.host.as_str(), args.port)).await?;
    let addr = server.local_addr()?;
    info!("Tcp server bound to {:?}", addr);

    let target = mitm::find_target().await?;
    info!("Found mitm target {:?}", target);

    let mut fake_server = target.clone();
    fake_server.address = addr;
    fake_server.name += "mitm";
    fake_server.peer_id = rand::random::<u64>();
    fake_server.uuid = Uuid::new_v4();

    let _registration = mdns::register(&fake_server).await?;
    info!("Registered as {:?}", fake_server);

    loop {
        let (stream, addr) = server.accept().await?;
        info!("New connection from {:?}", addr);
        let mitm = Mitm::new(target.clone(), fake_server.clone());
        tokio::spawn(async move {
            match handle_stream(stream, mitm).await {
                Ok(()) => info!("{:?} finished", addr),
                Err(e) => error!("Error occurred for {:?}: {:?}", addr, e),
            }
        });
    }
}
