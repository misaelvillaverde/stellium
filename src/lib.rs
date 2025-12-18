//! Stellium - Astrology MCP Server
//!
//! A Model Context Protocol server providing ephemeris data and astrological calculations
//! using the Swiss Ephemeris library.

pub mod ephemeris;
pub mod models;
pub mod server;
pub mod storage;

pub use server::StelliumServer;
