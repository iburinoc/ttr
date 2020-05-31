use std::net::Ipv4Addr;

use net2::UdpBuilder;
use structopt::StructOpt;
use tokio::net::UdpSocket;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "5000")]
    port: u16,

    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    #[structopt(short, long)]
    target: Option<String>,

    #[structopt(short, long)]
    msg: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opt::from_args();
    let mut listener = UdpSocket::from_std(
        UdpBuilder::new_v4()?
            .reuse_address(true)?
            .bind((opts.host.as_str(), opts.port))?,
    )?;

    listener.join_multicast_v4("224.0.0.251".parse()?, "192.168.2.45".parse()?)?;

    println!("Listening on {:?}", listener.local_addr()?);

    if let Some(msg) = opts.msg {
        if let Some(target) = opts.target {
            let bytes = msg.as_bytes();
            listener.connect(target).await?;
            listener.send(bytes).await?;
        }
    }

    let mut buf = [0u8; 2048];

    loop {
        let (amt, from) = listener.recv_from(&mut buf).await?;
        println!("Recv {:?}: {:02x?}", from, &buf[..amt]);
    }
}
