use clap::{Parser, command};
use futures::FutureExt;
use std::error::Error;
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "The port to listen")]
    port: u16,
    #[arg(short, long, help = "The list of backend servers")]
    servers: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    let args = Args::parse();
    debug!("parsed cli arguments: {:?}", args);
    let listen_addr = format!("127.0.0.1:{}", args.port);
    info!("Listening on: {listen_addr}");
    let listener = TcpListener::bind(listen_addr).await?;
    if let Some(server) = args.servers.first() {
        while let Ok((mut inbound, _)) = listener.accept().await {
            let mut outbound = TcpStream::connect(server.clone()).await?;
            tokio::spawn(async move {
                copy_bidirectional(&mut inbound, &mut outbound)
                    .map(|r| {
                        if let Err(e) = r {
                            error!("Failed to transfer; error={e}");
                        }
                    })
                    .await
            });
        }
    }
    Ok(())
}
