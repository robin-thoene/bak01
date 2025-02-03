use async_trait::async_trait;
use futures::FutureExt;
use std::error::Error;
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, info};

#[async_trait]
pub trait Proxy {
    fn new(port: u16, available_servers: Vec<String>) -> Self
    where
        Self: Sized;
    async fn run(&self) -> Result<(), Box<dyn Error>>;
}

pub struct UdpProxy {
    address: String,
    available_servers: Vec<String>,
}

pub struct TcpProxy {
    address: String,
    available_servers: Vec<String>,
}

#[async_trait]
impl Proxy for UdpProxy {
    fn new(port: u16, available_servers: Vec<String>) -> Self {
        UdpProxy {
            address: format!("127.0.0.1:{}", port),
            available_servers,
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        debug!(
            "Starting UDP mode with backends: {:?}",
            &self.available_servers
        );
        info!("Listening on: {}", &self.address);
        todo!();
    }
}

#[async_trait]
impl Proxy for TcpProxy {
    fn new(port: u16, available_servers: Vec<String>) -> Self {
        TcpProxy {
            address: format!("127.0.0.1:{}", port),
            available_servers,
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        debug!(
            "Starting TCP mode with backends: {:?}",
            &self.available_servers
        );
        info!("Listening on: {}", &self.address);
        let listener = TcpListener::bind(&self.address).await?;
        if let Some(server) = &self.available_servers.first() {
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
