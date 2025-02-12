use crate::load_balancer::LoadBalancer;
use async_trait::async_trait;
use futures::FutureExt;
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream, UdpSocket},
};
use tracing::{debug, error, info};

#[async_trait]
pub trait Proxy {
    fn new(port: u16, load_balancer: Arc<dyn LoadBalancer>) -> Self
    where
        Self: Sized;
    async fn run(&self) -> Result<(), Box<dyn Error>>;
}

pub struct UdpProxy {
    address: String,
    load_balancer: Arc<dyn LoadBalancer>,
}

pub struct TcpProxy {
    address: String,
    load_balancer: Arc<dyn LoadBalancer>,
}

#[async_trait]
impl Proxy for UdpProxy {
    fn new(port: u16, load_balancer: Arc<dyn LoadBalancer>) -> Self {
        Self {
            address: format!("127.0.0.1:{}", port),
            load_balancer,
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        debug!("Starting UDP mode");
        let proxy_socket = UdpSocket::bind(&self.address).await?;
        info!("Listening on: {}", &self.address);
        let mut to_send: Option<(usize, SocketAddr)> = None;
        let mut buf = vec![0; 1024];
        loop {
            if let Some((size, peer)) = to_send {
                let server = "127.0.0.1:8080";
                // Forward the data to the selected UDP server
                proxy_socket.send_to(&buf[..size], server).await?;
                // Receive the answer from the server
                let answer = proxy_socket.recv_from(&mut buf).await?;
                // Send the data back to the initial client
                proxy_socket.send_to(&buf[..answer.0], &peer).await?;
            }
            to_send = Some(proxy_socket.recv_from(&mut buf).await?);
        }
    }
}

#[async_trait]
impl Proxy for TcpProxy {
    fn new(port: u16, load_balancer: Arc<dyn LoadBalancer>) -> Self {
        Self {
            address: format!("127.0.0.1:{}", port),
            load_balancer,
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        debug!("Starting TCP mode");
        let listener = TcpListener::bind(&self.address).await?;
        info!("Listening on: {}", &self.address);
        while let Ok((mut inbound, _)) = listener.accept().await {
            let server = Arc::clone(&self.load_balancer).get_next_server();
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
        Ok(())
    }
}
