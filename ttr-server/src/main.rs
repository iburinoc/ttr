#![recursion_limit = "256"]

use std::error::Error as StdError;

use uuid::Uuid;

use structopt::StructOpt;

mod dummy;
mod mitm;
mod util;

#[derive(Debug, StructOpt)]
#[structopt(name = "ttr", about = "Ticket to ride app man in the middle server")]
pub enum Cmd {
    Mitm(MitmArgs),
    Dummy(DummyArgs),
}

#[derive(Debug, StructOpt)]
pub struct MitmArgs {
    /// Hostname to bind to
    #[structopt(short = "H", long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind to
    #[structopt(short, long, default_value = "0")]
    port: u16,

    #[structopt(flatten)]
    player_id: PlayerId,

    /// Path to write packet files to
    #[structopt(short = "l", long = "log-path")]
    log_path: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct DummyArgs {
    #[structopt(flatten)]
    player_id: PlayerId,

    /// Path to write packet files to
    #[structopt(short = "l", long = "log-path")]
    log_path: Option<String>,

    /// Name to play as
    #[structopt(short, long, default_value = "dummy")]
    name: String,
}

#[derive(Debug, StructOpt)]
pub struct PlayerId {
    /// What peer id to advertise as
    #[structopt(short = "P", long)]
    peer_id: Option<u64>,

    /// What UUID to advertise as
    #[structopt(short = "u", long)]
    uuid: Option<Uuid>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    env_logger::init_from_env(
        env_logger::Env::new().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = Cmd::from_args();

    match args {
        Cmd::Mitm(args) => mitm::run(args).await?,
        Cmd::Dummy(args) => dummy::run(args).await?,
    }

    Ok(())
}
