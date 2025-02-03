use axum::{Router, response::Html, routing::get};
use std::{error::Error, net::SocketAddr, str, time::Duration};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, UdpSocket},
    spawn,
    time::sleep,
};
use tracing::{error, info};

pub enum Protocol {
    Tcp,
    Udp,
    Http,
}

pub struct Server {
    protocol: Protocol,
    address: String,
}

impl Server {
    /// Initialize a new server
    ///
    /// # Arguments
    ///
    /// * `protocol` - The protocol to use
    /// * `port` - The port to use
    pub fn new(protocol: Protocol, port: u16) -> Self {
        Server {
            protocol,
            address: format!("127.0.0.1:{}", port),
        }
    }

    /// Runs the server based on the given configuration
    ///
    /// # Errors
    ///
    /// Any error that might occur
    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        match self.protocol {
            Protocol::Tcp => self.run_tcp_server().await,
            Protocol::Udp => self.run_udp_server().await,
            Protocol::Http => self.run_http_server().await,
        }
    }

    /// Runs a simple TCP server that keeps the connection
    /// alive and sends a message to the connected client
    /// in intervals
    ///
    /// # Errors
    ///
    /// Any error that might occur
    async fn run_tcp_server(&self) -> Result<(), Box<dyn Error>> {
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

    /// Runs a simple UDP server that responds to a clients
    /// message by sending it back
    ///
    /// # Errors
    ///
    /// Any error that might occur
    async fn run_udp_server(&self) -> Result<(), Box<dyn Error>> {
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

    /// Runs a simple HTTP server that responds
    /// with HTML
    async fn run_http_server(&self) -> Result<(), Box<dyn Error>> {
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
