//! Stellium MCP Server - Entry Point
//!
//! This binary provides an MCP server via STDIO transport for astrological calculations.

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use stellium::ephemeris::init_ephemeris;
use stellium::StelliumServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is used for MCP communication)
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting Stellium MCP Server");

    // Initialize Swiss Ephemeris
    init_ephemeris();

    // Create and run the MCP server
    let server = StelliumServer::new();
    let service = server.serve(stdio()).await?;

    tracing::info!("Server initialized, waiting for requests...");

    // Wait for shutdown
    service.waiting().await?;

    tracing::info!("Server shutting down");

    Ok(())
}
