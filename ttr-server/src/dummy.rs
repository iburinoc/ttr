use log::*;

use super::util;

pub async fn run(args: super::DummyArgs) -> anyhow::Result<()> {
    let server = util::find_server().await?;
    info!("Found server {:?}", server);

    Ok(())
}
