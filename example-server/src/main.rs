use axum::{Router, response::Html, routing::get};
use clap::{Parser, ValueEnum};
use std::{error::Error, net::SocketAddr, str, sync::Arc, time::Duration};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, UdpSocket},
    spawn,
    time::sleep,
};
use tracing::{debug, error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, ValueEnum)]
enum ServerType {
    Tcp,
    Udp,
    Http,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum, help = "The type of server to start")]
    server_type: ServerType,
    #[arg(short, long, help = "The port to listen")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    let args = Args::parse();
    debug!("parsed cli arguments: {:?}", args);
    let addr = format!("127.0.0.1:{}", args.port);
    match args.server_type {
        ServerType::Tcp => run_tcp_server(addr).await,
        ServerType::Udp => run_udp_server(addr).await,
        ServerType::Http => run_http_server(addr).await,
    }
}

/// Runs a simple TCP server that keeps the connection
/// alive and sends a message to the connected client
/// in intervals
///
/// # Arguments
///
/// * `addr` - The address on that the server shall listen
///
/// # Errors
///
/// Any error that might occur
async fn run_tcp_server(addr: String) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&addr).await?;
    info!(
        "Listening on: {}",
        listener
            .local_addr()
            .expect("Could not get local socket address")
    );
    loop {
        let (mut socket, _) = listener.accept().await?;
        spawn(async move {
            loop {
                socket
                    .write_all(b"Tic\n")
                    .await
                    .expect("failed to write data to socket");
                sleep(Duration::from_secs(1)).await;
            }
        });
    }
}

/// Runs a simple UDP server that responds to a clients
/// message by sending it back
///
/// # Arguments
///
/// * `addr` - The address on that the server shall expose
///
/// # Errors
///
/// Any error that might occur
async fn run_udp_server(addr: String) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(&addr).await?;
    info!(
        "Listening on: {}",
        socket
            .local_addr()
            .expect("Could not get local socket address")
    );
    let mut to_send: Option<(usize, SocketAddr)> = None;
    let mut buf = vec![0; 1024];
    loop {
        if let Some((size, peer)) = to_send {
            match str::from_utf8(&buf[..size]) {
                Ok(msg) => {
                    let res = socket
                        .send_to(format!("You sent: {}\n", msg).as_bytes(), &peer)
                        .await;
                    match res {
                        Ok(_) => {}
                        Err(err) => {
                            error!(
                                "Error sending message to client with address {}: {:?}",
                                peer, err
                            );
                        }
                    }
                }
                Err(err) => {
                    error!("An error occurred reading a users message: {:?}", err);
                    socket
                        .send_to("An error occurred reading you message\n".as_bytes(), &peer)
                        .await?;
                }
            };
        }
        to_send = Some(socket.recv_from(&mut buf).await?);
    }
}

/// Runs a simple HTTP server that responds
/// with HTML
///
/// # Arguments
///
/// * `addr` - The address on that the server shall expose
async fn run_http_server(addr: String) -> Result<(), Box<dyn Error>> {
    async fn root_handler() -> Html<&'static str> {
        Html("<h1>Hello, World!</h1>")
    }
    let app = Router::new().route("/", get(root_handler));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(
        "listening on {}",
        Arc::new(
            listener
                .local_addr()
                .expect("Could not get local socket address")
        )
    );
    axum::serve(listener, app).await?;
    Ok(())
}
