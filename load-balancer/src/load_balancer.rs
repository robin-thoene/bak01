use std::{
    net::SocketAddr,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
};
use tracing::debug;

pub trait LoadBalancer: Sync + Send {
    fn new(servers: Vec<String>) -> Self
    where
        Self: Sized;
    fn get_next_server(&self) -> SocketAddr;
    fn update_server(&self, server: SocketAddr);
}

pub struct RoundRobinLoadBalancer {
    servers: Vec<SocketAddr>,
    next_index: AtomicUsize,
}

pub struct LeastConnectionLoadBalancer {
    servers: Vec<(SocketAddr, AtomicU32)>,
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

    fn update_server(&self, server: SocketAddr) {
        // The round robin algorithm does not care about server state changes
        debug!(
            "Skipping update of server {}, round robin has nothing to do",
            server.to_string()
        )
    }
}

impl LoadBalancer for LeastConnectionLoadBalancer {
    fn new(servers: Vec<String>) -> Self {
        let server_socket_addr = parse_server_addresses_from_string(servers);
        let servers = server_socket_addr
            .iter()
            .map(|x| (*x, AtomicU32::new(0)))
            .collect::<Vec<(SocketAddr, AtomicU32)>>();
        Self { servers }
    }

    fn get_next_server(&self) -> SocketAddr {
        let server = self
            .servers
            .iter()
            .min_by_key(|x| x.1.load(Ordering::Relaxed))
            .expect("At least one server must always be configured");
        debug!(
            "Current servers: {:?},  forwarding req to {}",
            self.servers,
            server.0.to_string()
        );
        // Ensure that the new connection is being tracked
        server.1.fetch_add(1, Ordering::SeqCst);
        server.0
    }

    fn update_server(&self, server: SocketAddr) {
        let server_mut = self.servers.iter().find(|x| x.0 == server);
        if let Some(server_mut) = server_mut {
            debug!("Remove one connection from server {}", server.to_string());
            server_mut.1.fetch_sub(1, Ordering::SeqCst);
        }
    }
}
