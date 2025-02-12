use std::{
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
};

use tracing::debug;

pub trait LoadBalancer: Sync + Send {
    fn new(servers: Vec<String>) -> Self
    where
        Self: Sized;
    fn get_next_server(&self) -> SocketAddr;
}

pub struct RoundRobinLoadBalancer {
    servers: Vec<SocketAddr>,
    next_index: AtomicUsize,
}

pub struct LeastConnectionLoadBalancer {
    servers: Vec<SocketAddr>,
}

///
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

impl LoadBalancer for RoundRobinLoadBalancer {
    fn new(servers: Vec<String>) -> Self {
        Self {
            servers: parse_server_addresses_from_string(servers),
            next_index: AtomicUsize::new(0),
        }
    }

    fn get_next_server(&self) -> SocketAddr {
        let i = self.next_index.fetch_add(1, Ordering::SeqCst) % self.servers.len();
        let server = *self
            .servers
            .get(i)
            .expect("Expect server in vec to be present");

        debug!(
            "Choose server at index {}, forwarding req to {}",
            i,
            server.to_string()
        );
        server
    }
}

impl LoadBalancer for LeastConnectionLoadBalancer {
    fn new(servers: Vec<String>) -> Self {
        Self {
            servers: parse_server_addresses_from_string(servers),
        }
    }

    fn get_next_server(&self) -> SocketAddr {
        *self
            .servers
            .first()
            .expect("At least one server must always be configured")
    }
}
