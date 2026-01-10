use dashmap::DashMap;
use tokio::sync::mpsc;

/// Server state managing connected clients
pub struct ServerState {
    /// user_id -> list of sender channels (supports multiple connections per user)
    pub clients: DashMap<String, Vec<mpsc::UnboundedSender<String>>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
        }
    }

    /// Register a new client connection (supports multiple connections per user)
    pub fn add_client(&self, user_id: String, tx: mpsc::UnboundedSender<String>) {
        self.clients
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(tx);
    }

    /// Remove a specific client connection by checking if the channel is closed
    pub fn remove_client(&self, user_id: &str) {
        if let Some(mut entry) = self.clients.get_mut(user_id) {
            // Remove closed channels
            entry.retain(|tx| !tx.is_closed());
            // If no channels left, remove the user entry
            if entry.is_empty() {
                drop(entry);
                self.clients.remove(user_id);
            }
        }
    }

    /// Broadcast message to all clients except the sender
    pub fn broadcast(&self, message: &str, exclude_user_id: Option<&str>) {
        for entry in self.clients.iter() {
            if Some(entry.key().as_str()) != exclude_user_id {
                for tx in entry.value().iter() {
                    let _ = tx.send(message.to_string());
                }
            }
        }
    }

    /// Send message to a specific user (sends to all their connections)
    pub fn send_to_user(&self, user_id: &str, message: &str) -> bool {
        if let Some(channels) = self.clients.get(user_id) {
            let mut sent = false;
            for tx in channels.iter() {
                if tx.send(message.to_string()).is_ok() {
                    sent = true;
                }
            }
            sent
        } else {
            false
        }
    }

    /// Get list of online user IDs
    pub fn online_users(&self) -> Vec<String> {
        self.clients
            .iter()
            .filter(|e| !e.value().is_empty())
            .map(|e| e.key().clone())
            .collect()
    }

    /// Check if a user is online
    pub fn is_online(&self, user_id: &str) -> bool {
        self.clients
            .get(user_id)
            .map(|channels| !channels.is_empty())
            .unwrap_or(false)
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_server_state() {
        let state = ServerState::new();
        assert!(state.clients.is_empty());
        assert!(state.online_users().is_empty());
    }

    #[test]
    fn test_add_and_remove_client() {
        let state = ServerState::new();
        let (tx, rx) = mpsc::unbounded_channel();

        state.add_client("user1".to_string(), tx);
        assert!(state.is_online("user1"));
        assert_eq!(state.online_users().len(), 1);

        // Drop rx to close the channel, then remove_client will clean it up
        drop(rx);
        state.remove_client("user1");
        assert!(!state.is_online("user1"));
        assert!(state.online_users().is_empty());
    }

    #[test]
    fn test_multiple_clients() {
        let state = ServerState::new();
        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();
        let (tx3, _rx3) = mpsc::unbounded_channel();

        state.add_client("user1".to_string(), tx1);
        state.add_client("user2".to_string(), tx2);
        state.add_client("user3".to_string(), tx3);

        assert_eq!(state.online_users().len(), 3);
        assert!(state.is_online("user1"));
        assert!(state.is_online("user2"));
        assert!(state.is_online("user3"));
        assert!(!state.is_online("user4"));
    }

    #[test]
    fn test_send_to_user() {
        let state = ServerState::new();
        let (tx, mut rx) = mpsc::unbounded_channel();

        state.add_client("user1".to_string(), tx);

        // Send to existing user
        assert!(state.send_to_user("user1", "hello"));

        // Verify message received
        let msg = rx.try_recv().unwrap();
        assert_eq!(msg, "hello");

        // Send to non-existing user
        assert!(!state.send_to_user("user2", "hello"));
    }

    #[test]
    fn test_broadcast_excludes_sender() {
        let state = ServerState::new();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();
        let (tx3, mut rx3) = mpsc::unbounded_channel();

        state.add_client("user1".to_string(), tx1);
        state.add_client("user2".to_string(), tx2);
        state.add_client("user3".to_string(), tx3);

        // Broadcast excluding user1
        state.broadcast("test message", Some("user1"));

        // user1 should NOT receive the message
        assert!(rx1.try_recv().is_err());

        // user2 and user3 should receive it
        assert_eq!(rx2.try_recv().unwrap(), "test message");
        assert_eq!(rx3.try_recv().unwrap(), "test message");
    }

    #[test]
    fn test_broadcast_to_all() {
        let state = ServerState::new();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();

        state.add_client("user1".to_string(), tx1);
        state.add_client("user2".to_string(), tx2);

        // Broadcast to all (None excludes nobody)
        state.broadcast("global message", None);

        // Both should receive it
        assert_eq!(rx1.try_recv().unwrap(), "global message");
        assert_eq!(rx2.try_recv().unwrap(), "global message");
    }

    #[test]
    fn test_multiple_connections_per_user() {
        let state = ServerState::new();
        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();

        // Add same user with two connections (frontend + backend scenario)
        state.add_client("user1".to_string(), tx1);
        state.add_client("user1".to_string(), tx2);

        // Should still show as one online user
        assert_eq!(state.online_users().len(), 1);
        assert!(state.is_online("user1"));

        // Send message - both connections should receive it
        state.send_to_user("user1", "hello");
        assert_eq!(rx1.try_recv().unwrap(), "hello");
        assert_eq!(rx2.try_recv().unwrap(), "hello");
    }

    #[test]
    fn test_partial_disconnect() {
        let state = ServerState::new();
        let (tx1, rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();

        // Add same user with two connections
        state.add_client("user1".to_string(), tx1);
        state.add_client("user1".to_string(), tx2);

        // Close first connection
        drop(rx1);
        state.remove_client("user1");

        // User should still be online via second connection
        assert!(state.is_online("user1"));

        // Second connection should still receive messages
        assert!(state.send_to_user("user1", "still connected"));
        assert_eq!(rx2.try_recv().unwrap(), "still connected");
    }

    #[test]
    fn test_default_impl() {
        let state = ServerState::default();
        assert!(state.clients.is_empty());
    }
}
