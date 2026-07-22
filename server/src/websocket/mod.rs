//! WebSocket server for real-time events.
//!
//! Exposes an authenticated `/websocket` endpoint, tracks active connections in
//! memory, and broadcasts message, reaction, typing, and presence events.

pub mod bus;
pub mod handler;
pub mod manager;

pub use bus::{WebSocketEventBus, WebSocketEventBusDeps};
pub use handler::websocket_handler;
pub use manager::{ConnectionId, ConnectionManager};
