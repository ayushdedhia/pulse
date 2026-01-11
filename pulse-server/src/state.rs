use dashmap::DashMap;
use tokio::sync::mpsc;
use tracing::info;

/// Maximum pending messages per user to prevent unbounded memory growth
const MAX_PENDING_MESSAGES_PER_USER: usize = 1000;

/// Server state managing connected clients and pending messages
pub struct ServerState {
    /// user_id -> list of sender channels (supports multiple connections per user)
    pub clients: DashMap<String, Vec<mpsc::UnboundedSender<String>>>,
    /// user_id -> list of pending messages (for offline users)
    pending_messages: DashMap<String, Vec<String>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
            pending_messages: DashMap::new(),
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

    /// Queue a message for an offline user
    pub fn queue_message(&self, user_id: &str, message: String) {
        let mut entry = self
            .pending_messages
            .entry(user_id.to_string())
            .or_insert_with(Vec::new);

        // Enforce queue limit - drop oldest if at capacity
        if entry.len() >= MAX_PENDING_MESSAGES_PER_USER {
            entry.remove(0);
            info!(
                "Queue limit reached for {}, dropped oldest message",
                user_id
            );
        }
        entry.push(message);
    }

    /// Take all pending messages for a user (clears the queue)
    pub fn take_pending_messages(&self, user_id: &str) -> Vec<String> {
        self.pending_messages
            .remove(user_id)
            .map(|(_, msgs)| msgs)
            .unwrap_or_default()
    }

    /// Send to user if online, otherwise queue the message
    /// Returns true if sent immediately, false if queued
    pub fn send_or_queue(&self, user_id: &str, message: &str) -> bool {
        if self.send_to_user(user_id, message) {
            true
        } else {
            self.queue_message(user_id, message.to_string());
            info!("Queued message for offline user {}", user_id);
            false
        }
    }

    /// Get the number of pending messages for a user
    pub fn pending_count(&self, user_id: &str) -> usize {
        self.pending_messages
            .get(user_id)
            .map(|msgs| msgs.len())
            .unwrap_or(0)
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

    #[test]
    fn test_queue_message_stores_correctly() {
        let state = ServerState::new();

        state.queue_message("user1", "message1".to_string());
        state.queue_message("user1", "message2".to_string());

        assert_eq!(state.pending_count("user1"), 2);
        assert_eq!(state.pending_count("user2"), 0);
    }

    #[test]
    fn test_take_pending_messages_clears_queue() {
        let state = ServerState::new();

        state.queue_message("user1", "msg1".to_string());
        state.queue_message("user1", "msg2".to_string());
        state.queue_message("user1", "msg3".to_string());

        let messages = state.take_pending_messages("user1");
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0], "msg1");
        assert_eq!(messages[1], "msg2");
        assert_eq!(messages[2], "msg3");

        // Queue should be empty now
        assert_eq!(state.pending_count("user1"), 0);
        assert!(state.take_pending_messages("user1").is_empty());
    }

    #[test]
    fn test_send_or_queue_routes_correctly() {
        let state = ServerState::new();
        let (tx, mut rx) = mpsc::unbounded_channel();

        // User is offline - should queue
        assert!(!state.send_or_queue("offline_user", "queued msg"));
        assert_eq!(state.pending_count("offline_user"), 1);

        // User comes online
        state.add_client("online_user".to_string(), tx);

        // User is online - should send immediately
        assert!(state.send_or_queue("online_user", "direct msg"));
        assert_eq!(rx.try_recv().unwrap(), "direct msg");
        assert_eq!(state.pending_count("online_user"), 0);
    }

    #[test]
    fn test_queue_limit_drops_oldest() {
        let state = ServerState::new();

        // Fill queue to limit
        for i in 0..MAX_PENDING_MESSAGES_PER_USER {
            state.queue_message("user1", format!("msg{}", i));
        }
        assert_eq!(state.pending_count("user1"), MAX_PENDING_MESSAGES_PER_USER);

        // Add one more - should drop oldest
        state.queue_message("user1", "new_msg".to_string());
        assert_eq!(state.pending_count("user1"), MAX_PENDING_MESSAGES_PER_USER);

        // Verify oldest was dropped and newest is present
        let messages = state.take_pending_messages("user1");
        assert_eq!(messages[0], "msg1"); // msg0 was dropped
        assert_eq!(messages[messages.len() - 1], "new_msg");
    }

    #[test]
    fn test_pending_messages_per_user_isolation() {
        let state = ServerState::new();

        state.queue_message("user1", "user1_msg".to_string());
        state.queue_message("user2", "user2_msg".to_string());

        assert_eq!(state.pending_count("user1"), 1);
        assert_eq!(state.pending_count("user2"), 1);

        let user1_msgs = state.take_pending_messages("user1");
        assert_eq!(user1_msgs.len(), 1);
        assert_eq!(user1_msgs[0], "user1_msg");

        // user2's messages should be unaffected
        assert_eq!(state.pending_count("user2"), 1);
    }
}
