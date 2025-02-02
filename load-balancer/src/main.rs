use clap::{Parser, ValueEnum, command};
use futures::FutureExt;
use std::error::Error;
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, ValueEnum)]
enum ProxyType {
    Udp,
    Tcp,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "The port to listen")]
    port: u16,
    #[arg(short, long, help = "The list of backend servers")]
    servers: Vec<String>,
    #[arg(
        short = 't',
        long,
        value_enum,
        help = "The protocol to use for the proxy"
    )]
    proxy_type: ProxyType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    let args = Args::parse();
    debug!("parsed cli arguments: {:?}", args);
    let listener_addr = format!("127.0.0.1:{}", args.port);
    info!("Listening on: {listener_addr}");
    match args.proxy_type {
        ProxyType::Udp => {}
        ProxyType::Tcp => run_with_tcp(listener_addr, args.servers).await?,
    }
    Ok(())
}

/// Runs the load balancer using the TCP protocol
///
/// # Arguments
///
/// * `listener_addr` - The address on which the load balancer listens for incoming requests
/// * `available_servers` - The list of backend servers the load balancer can forward requests to
///
/// # Errors
///
/// Any occurred error
async fn run_with_tcp(
    listener_addr: String,
    available_servers: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(listener_addr).await?;
    if let Some(server) = available_servers.first() {
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
