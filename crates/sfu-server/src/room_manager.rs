use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct Participant {
    pub id: String,
    // Future: webrtc::peer_connection::RTCPeerConnection
}

pub struct Room {
    pub id: String,
    pub participants: HashMap<String, Arc<Participant>>,
}

pub struct RoomManager {
    pub rooms: Arc<RwLock<HashMap<String, Room>>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn join_room(&self, room_id: String, participant_id: String) {
        let mut rooms = self.rooms.write().await;
        let room = rooms.entry(room_id.clone()).or_insert_with(|| {
            info!("Creating new room on server: {}", room_id);
            Room {
                id: room_id.clone(),
                participants: HashMap::new(),
            }
        });

        room.participants.insert(
            participant_id.clone(),
            Arc::new(Participant { id: participant_id }),
        );
    }

    pub async fn leave_room(&self, room_id: &str, participant_id: &str) {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.participants.remove(participant_id);
            if room.participants.is_empty() {
                info!("Room {} is empty, removing", room_id);
                rooms.remove(room_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_manager_creation() {
        let manager = RoomManager::new();
        // Room manager should initialize successfully
        assert!(manager.rooms.try_read().is_ok());
    }

    #[test]
    fn test_room_creation() {
        let room = Room {
            id: "test-room".to_string(),
            participants: HashMap::new(),
        };
        assert_eq!(room.id, "test-room");
        assert!(room.participants.is_empty());
    }

    #[test]
    fn test_participant_creation() {
        let participant = Participant {
            id: "participant-1".to_string(),
        };
        assert_eq!(participant.id, "participant-1");
    }

    #[tokio::test]
    async fn test_join_room() {
        let manager = RoomManager::new();
        manager
            .join_room("room1".to_string(), "participant1".to_string())
            .await;

        let rooms = manager.rooms.read().await;
        assert!(rooms.contains_key("room1"));
    }

    #[tokio::test]
    async fn test_leave_room() {
        let manager = RoomManager::new();

        // Join first
        manager
            .join_room("room1".to_string(), "participant1".to_string())
            .await;

        // Then leave
        manager.leave_room("room1", "participant1").await;

        // Room should be removed when empty
        let rooms = manager.rooms.read().await;
        assert!(!rooms.contains_key("room1"));
    }

    #[tokio::test]
    async fn test_multiple_participants() {
        let manager = RoomManager::new();

        manager
            .join_room("room1".to_string(), "p1".to_string())
            .await;
        manager
            .join_room("room1".to_string(), "p2".to_string())
            .await;
        manager
            .join_room("room1".to_string(), "p3".to_string())
            .await;

        let rooms = manager.rooms.read().await;
        if let Some(room) = rooms.get("room1") {
            assert_eq!(room.participants.len(), 3);
        }
    }

    #[tokio::test]
    async fn test_room_cleanup_when_empty() {
        let manager = RoomManager::new();

        // Add and remove participants
        manager
            .join_room("room1".to_string(), "p1".to_string())
            .await;
        manager
            .join_room("room1".to_string(), "p2".to_string())
            .await;

        manager.leave_room("room1", "p1").await;
        manager.leave_room("room1", "p2").await;

        // Room should be automatically removed
        let rooms = manager.rooms.read().await;
        assert!(!rooms.contains_key("room1"));
    }
}
