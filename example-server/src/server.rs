use async_trait::async_trait;
use axum::{Router, response::Html, routing::get};
use std::{error::Error, net::SocketAddr, str, time::Duration};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, UdpSocket},
    spawn,
    time::sleep,
};
use tracing::{error, info};

#[async_trait]
pub trait Serverable {
    fn new(port: u16) -> Self
    where
        Self: Sized;
    async fn run(&self) -> Result<(), Box<dyn Error>>;
}

pub struct UdpServer {
    address: String,
}

pub struct TcpServer {
    address: String,
}

pub struct HttpServer {
    address: String,
}

#[async_trait]
impl Serverable for UdpServer {
    fn new(port: u16) -> Self {
        UdpServer {
            address: format!("127.0.0.1:{}", port),
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let socket = UdpSocket::bind(&self.address).await?;
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
}

#[async_trait]
impl Serverable for TcpServer {
    fn new(port: u16) -> Self {
        TcpServer {
            address: format!("127.0.0.1:{}", port),
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.address).await?;
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
}

#[async_trait]
impl Serverable for HttpServer {
    fn new(port: u16) -> Self {
        HttpServer {
            address: format!("127.0.0.1:{}", port),
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        async fn root_handler() -> Html<&'static str> {
            Html("<h1>Hello, World!</h1>")
        }
        let app = Router::new().route("/", get(root_handler));
        let listener = tokio::net::TcpListener::bind(&self.address).await?;
        info!(
            "listening on {}",
            listener
                .local_addr()
                .expect("Could not get local socket address")
        );
        axum::serve(listener, app).await?;
        Ok(())
    }
}
