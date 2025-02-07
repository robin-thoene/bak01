use async_trait::async_trait;
use futures::FutureExt;
use std::{error::Error, net::SocketAddr};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream, UdpSocket},
};
use tracing::{debug, error, info};

#[async_trait]
pub trait Proxy {
    fn new(port: u16, servers: Vec<String>) -> Self
    where
        Self: Sized;
    async fn run(&self) -> Result<(), Box<dyn Error>>;
}

pub struct UdpProxy {
    address: String,
    servers: Vec<SocketAddr>,
}

pub struct TcpProxy {
    address: String,
    servers: Vec<SocketAddr>,
}

/// Parses a list of server addresses in string format to the needed
/// datatype to run a proxy server.
///
/// # Arguments
///
/// * `servers` - The list of servers in string format, including IP and port
///
/// # Panics
///
/// When at least one server address can not be parsed to a socket address, the program should
/// not start because a list of valid server socket addresses is mandatory.
fn parse_server_addresses_from_string(servers: Vec<String>) -> Vec<SocketAddr> {
    servers
        .iter()
        .map(|s| {
            s.parse::<SocketAddr>()
                .unwrap_or_else(|_| panic!("The provided server {} could not be parsed", s))
        })
        .collect::<Vec<SocketAddr>>()
}

#[async_trait]
impl Proxy for UdpProxy {
    fn new(port: u16, servers: Vec<String>) -> Self {
        UdpProxy {
            address: format!("127.0.0.1:{}", port),
            servers: parse_server_addresses_from_string(servers),
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        debug!("Starting UDP mode with backends: {:?}", &self.servers);
        let proxy_socket = UdpSocket::bind(&self.address).await?;
        info!("Listening on: {}", &self.address);
        let mut to_send: Option<(usize, SocketAddr)> = None;
        let mut buf = vec![0; 1024];
        loop {
            if let Some((size, peer)) = to_send {
                if let Some(server) = &self.servers.first() {
                    // Forward the data to the selected UDP server
                    proxy_socket.send_to(&buf[..size], server).await?;
                    // Receive the answer from the server
                    let answer = proxy_socket.recv_from(&mut buf).await?;
                    // Send the data back to the initial client
                    proxy_socket.send_to(&buf[..answer.0], &peer).await?;
                }
            }
            to_send = Some(proxy_socket.recv_from(&mut buf).await?);
        }
    }
}

#[async_trait]
impl Proxy for TcpProxy {
    fn new(port: u16, servers: Vec<String>) -> Self {
        TcpProxy {
            address: format!("127.0.0.1:{}", port),
            servers: parse_server_addresses_from_string(servers),
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        debug!("Starting TCP mode with backends: {:?}", &self.servers);
        let listener = TcpListener::bind(&self.address).await?;
        info!("Listening on: {}", &self.address);
        if let Some(server) = &self.servers.first() {
            while let Ok((mut inbound, _)) = listener.accept().await {
                let mut outbound = TcpStream::connect(server).await?;
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
}
