pub mod clap;
pub mod server;

use ::clap::Parser;
use std::error::Error;
use tracing::{debug, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use crate::clap::CliArgs;
use crate::server::{Protocol, Server};

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
    let server = Server::new(Protocol::from(args.server_type), args.port);
    // Run the server with the desired configuration
    server.run().await
}
