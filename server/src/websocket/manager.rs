//! In-memory WebSocket connection registry and broadcaster.
//!
//! The manager tracks active sockets per user and organization, dispatching
//! events to the relevant connections. All state lives in the server process;
//! there is no external broker.

use crate::services::events::EventEnvelope;
use ruckchat_id::{OrganizationId, UserId};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

/// Identifier for a single WebSocket connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    /// Generates a new connection identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to a registered connection.
struct Connection {
    /// User owning the connection.
    user_id: UserId,
    /// Sender for pushing events to the socket task.
    sender: mpsc::UnboundedSender<EventEnvelope>,
    /// Organizations this connection receives broadcasts for.
    subscribed_organizations: HashSet<OrganizationId>,
}

/// In-memory registry of active WebSocket connections.
#[derive(Clone, Default)]
pub struct ConnectionManager {
    inner: Arc<RwLock<Registry>>,
}

impl std::fmt::Debug for ConnectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionManager").finish_non_exhaustive()
    }
}

#[derive(Default)]
struct Registry {
    /// Connections indexed by id.
    connections: HashMap<ConnectionId, Connection>,
    /// Connection ids grouped by user.
    by_user: HashMap<UserId, HashSet<ConnectionId>>,
    /// Connection ids grouped by subscribed organization.
    by_organization: HashMap<OrganizationId, HashSet<ConnectionId>>,
}

impl ConnectionManager {
    /// Creates an empty connection manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new connection for a user and returns its id.
    pub async fn register(
        &self,
        user_id: UserId,
        sender: mpsc::UnboundedSender<EventEnvelope>,
    ) -> ConnectionId {
        let connection_id = ConnectionId::new();
        let mut registry = self.inner.write().await;
        registry.connections.insert(
            connection_id,
            Connection {
                user_id,
                sender,
                subscribed_organizations: HashSet::new(),
            },
        );
        registry
            .by_user
            .entry(user_id)
            .or_default()
            .insert(connection_id);
        connection_id
    }

    /// Unregisters a connection and removes all subscriptions.
    pub async fn unregister(&self, connection_id: ConnectionId) {
        let mut registry = self.inner.write().await;
        let Some(connection) = registry.connections.remove(&connection_id) else {
            return;
        };
        if let Some(set) = registry.by_user.get_mut(&connection.user_id) {
            set.remove(&connection_id);
            if set.is_empty() {
                registry.by_user.remove(&connection.user_id);
            }
        }
        for organization_id in &connection.subscribed_organizations {
            if let Some(set) = registry.by_organization.get_mut(organization_id) {
                set.remove(&connection_id);
                if set.is_empty() {
                    registry.by_organization.remove(organization_id);
                }
            }
        }
    }

    /// Subscribes a connection to an organization.
    pub async fn subscribe_organization(
        &self,
        connection_id: ConnectionId,
        organization_id: OrganizationId,
    ) {
        let mut registry = self.inner.write().await;
        let Some(connection) = registry.connections.get_mut(&connection_id) else {
            return;
        };
        if connection.subscribed_organizations.insert(organization_id) {
            registry
                .by_organization
                .entry(organization_id)
                .or_default()
                .insert(connection_id);
        }
    }

    /// Unsubscribes a connection from an organization.
    pub async fn unsubscribe_organization(
        &self,
        connection_id: ConnectionId,
        organization_id: OrganizationId,
    ) {
        let mut registry = self.inner.write().await;
        let Some(connection) = registry.connections.get_mut(&connection_id) else {
            return;
        };
        if !connection.subscribed_organizations.remove(&organization_id) {
            return;
        }
        if let Some(set) = registry.by_organization.get_mut(&organization_id) {
            set.remove(&connection_id);
            if set.is_empty() {
                registry.by_organization.remove(&organization_id);
            }
        }
    }

    /// Returns the number of open connections for a user.
    pub async fn connection_count_for_user(&self, user_id: UserId) -> usize {
        let registry = self.inner.read().await;
        registry.by_user.get(&user_id).map_or(0, HashSet::len)
    }

    /// Sends an event to a single connection.
    pub async fn send(&self, connection_id: ConnectionId, envelope: EventEnvelope) {
        let registry = self.inner.read().await;
        if let Some(connection) = registry.connections.get(&connection_id) {
            let _ = connection.sender.send(envelope);
        }
    }

    /// Broadcasts an event to every connection subscribed to an organization.
    pub async fn broadcast_to_organization(
        &self,
        organization_id: OrganizationId,
        envelope: EventEnvelope,
    ) {
        let registry = self.inner.read().await;
        if let Some(ids) = registry.by_organization.get(&organization_id) {
            for id in ids.clone() {
                if let Some(connection) = registry.connections.get(&id) {
                    let _ = connection.sender.send(envelope.clone());
                }
            }
        }
    }

    /// Broadcasts an event to every open connection for a set of users.
    pub async fn broadcast_to_users(&self, user_ids: &[UserId], envelope: EventEnvelope) {
        let registry = self.inner.read().await;
        let mut seen = HashSet::new();
        for user_id in user_ids {
            if let Some(ids) = registry.by_user.get(user_id) {
                for id in ids.clone() {
                    if !seen.insert(id) {
                        continue;
                    }
                    if let Some(connection) = registry.connections.get(&id) {
                        let _ = connection.sender.send(envelope.clone());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::events::{PresenceStatus, ServerEvent};
    use ruckchat_id::{OrganizationId, UserId};

    fn envelope() -> EventEnvelope {
        EventEnvelope::new(ServerEvent::Presence {
            user_id: UserId::new(),
            status: PresenceStatus::Online,
        })
    }

    #[tokio::test]
    async fn registered_connection_receives_broadcast() {
        let manager = ConnectionManager::new();
        let user_id = UserId::new();
        let org_id = OrganizationId::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let conn_id = manager.register(user_id, tx).await;
        manager.subscribe_organization(conn_id, org_id).await;

        manager.broadcast_to_organization(org_id, envelope()).await;

        assert!(rx.try_recv().is_ok());
    }

    #[tokio::test]
    async fn unsubscribed_connection_does_not_receive_broadcast() {
        let manager = ConnectionManager::new();
        let user_id = UserId::new();
        let org_id = OrganizationId::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let conn_id = manager.register(user_id, tx).await;

        manager.broadcast_to_organization(org_id, envelope()).await;

        assert!(rx.try_recv().is_err());
        manager.subscribe_organization(conn_id, org_id).await;
        manager.broadcast_to_organization(org_id, envelope()).await;
        assert!(rx.try_recv().is_ok());
    }

    #[tokio::test]
    async fn unregister_removes_connection() {
        let manager = ConnectionManager::new();
        let user_id = UserId::new();
        let org_id = OrganizationId::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let conn_id = manager.register(user_id, tx).await;
        manager.subscribe_organization(conn_id, org_id).await;
        manager.unregister(conn_id).await;

        manager.broadcast_to_organization(org_id, envelope()).await;

        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn connection_count_tracks_user_connections() {
        let manager = ConnectionManager::new();
        let user_id = UserId::new();
        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();

        assert_eq!(manager.connection_count_for_user(user_id).await, 0);
        let conn1 = manager.register(user_id, tx1).await;
        assert_eq!(manager.connection_count_for_user(user_id).await, 1);
        let conn2 = manager.register(user_id, tx2).await;
        assert_eq!(manager.connection_count_for_user(user_id).await, 2);
        manager.unregister(conn1).await;
        assert_eq!(manager.connection_count_for_user(user_id).await, 1);
        manager.unregister(conn2).await;
        assert_eq!(manager.connection_count_for_user(user_id).await, 0);
    }

    #[tokio::test]
    async fn broadcast_to_users_targets_all_user_connections() {
        let manager = ConnectionManager::new();
        let user_id = UserId::new();
        let other_id = UserId::new();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        manager.register(user_id, tx1).await;
        manager.register(user_id, tx2).await;
        let (tx3, mut rx3) = mpsc::unbounded_channel();
        manager.register(other_id, tx3).await;

        manager
            .broadcast_to_users(
                &[user_id],
                EventEnvelope::new(ServerEvent::Presence {
                    user_id,
                    status: PresenceStatus::Offline,
                }),
            )
            .await;

        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
        assert!(rx3.try_recv().is_err());
    }
}
