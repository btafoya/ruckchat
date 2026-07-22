//! Model Context Protocol (MCP) server integration.
//!
//! Exposes a Streamable HTTP / SSE MCP endpoint authenticated via the same
//! session cookie or bearer token as the REST API. The MCP layer delegates
//! every tool call and resource read to the existing service layer so that
//! authorization rules remain consistent across transports.

pub mod handler;
pub mod resources;
pub mod server;
pub mod tools;

pub use handler::{McpHttpService, mcp_handler};
