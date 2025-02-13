pub mod clap;
pub mod load_balancer;
pub mod reverse_proxy;

use ::clap::Parser;
use clap::{CliArgs, LoadBalancerType, ProxyType};
use load_balancer::{LeastConnectionLoadBalancer, LoadBalancer, RoundRobinLoadBalancer};
use reverse_proxy::{Proxy, TcpProxy, UdpProxy};
use std::{error::Error, sync::Arc};
use tracing::{debug, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

/// Runs the application
///
/// # Errors
///
/// Any error that might occur
pub async fn run() -> Result<(), Box<dyn Error>> {
    // Set log and trace level
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();
    // Parse arguments and build config
    let args = CliArgs::parse();
    debug!("parsed cli arguments: {:?}", args);
    // Ensure that the CLI arguments are valid
    if args.servers.is_empty() {
        panic!("At least one backend server must be provided");
    }
    if args.proxy_type == ProxyType::Udp
        && args.load_balancer_type == LoadBalancerType::LeastConnection
    {
        panic!(
            "The load balancer type {:?} is not valid for {:?} proxies.",
            LoadBalancerType::LeastConnection,
            ProxyType::Udp
        );
    }
    // Run the desired proxy type
    let lb: Arc<dyn LoadBalancer> = match args.load_balancer_type {
        LoadBalancerType::RoundRobin => Arc::new(RoundRobinLoadBalancer::new(args.servers)),
        LoadBalancerType::LeastConnection => {
            Arc::new(LeastConnectionLoadBalancer::new(args.servers))
        }
    };
    let proxy: Box<dyn Proxy> = match args.proxy_type {
        ProxyType::Udp => Box::new(UdpProxy::new(args.port, lb)),
        ProxyType::Tcp => Box::new(TcpProxy::new(args.port, lb)),
    };
    proxy.run().await
}
