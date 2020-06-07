use std::error::Error as StdError;

use log::*;
use structopt::StructOpt;
use tokio::net::{TcpListener, TcpStream};

use ttr_net::{
    connection::Connection,
    mdns::{self, Server},
};

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

async fn handle_stream(stream: TcpStream, server: &Server, mitm: &Mitm) -> anyhow::Result<()> {
    let (_connection, receiver, sender) = Connection::from_stream(stream);
    mitm.run(server, receiver, sender).await
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

    let mitm = Mitm::new().await?;
    info!("Found mitm target {:?}", mitm.target());

    let mut fake_server = mitm.target().clone();
    fake_server.address = addr;
    fake_server.name += "mitm";
    fake_server.peer_id = rand::random::<u64>();

    let _registration = mdns::register(&fake_server).await?;
    info!("Registered as {:?}", fake_server);

    loop {
        let (stream, addr) = server.accept().await?;
        info!("New connection from {:?}", addr);
        let fake_server = fake_server.clone();
        let mitm = Mitm::from_target(mitm.target().clone());
        tokio::spawn(async move {
            match handle_stream(stream, &fake_server, &mitm).await {
                Ok(()) => info!("{:?} finished", addr),
                Err(e) => error!("Error occurred for {:?}: {:?}", addr, e),
            }
        });
    }
}
