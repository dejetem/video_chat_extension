use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use webrtc::peer_connection::RTCPeerConnection;

pub struct Participant {
    pub id: String,
    pub peer_connection: std::sync::Mutex<Option<Arc<RTCPeerConnection>>>,
    pub published_tracks: Vec<String>,
    pub sender: Option<tokio::sync::mpsc::UnboundedSender<video_chat_signaling::Message>>,
}

pub struct Room {
    pub id: String,
    pub participants: HashMap<String, Arc<Participant>>,
}

pub struct RoomManager {
    pub rooms: Arc<RwLock<HashMap<String, Room>>>,
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn join_room(
        &self,
        room_id: String,
        participant_id: String,
        sender: Option<tokio::sync::mpsc::UnboundedSender<video_chat_signaling::Message>>,
    ) {
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
            Arc::new(Participant {
                id: participant_id,
                peer_connection: std::sync::Mutex::new(None),
                published_tracks: Vec::new(),
                sender,
            }),
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

    pub async fn broadcast_to_room(&self, room_id: &str, message: video_chat_signaling::Message) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            for participant in room.participants.values() {
                if let Some(sender) = &participant.sender {
                    let _ = sender.send(message.clone());
                }
            }
        }
    }

    pub async fn broadcast_to_others(
        &self,
        room_id: &str,
        exclude_participant_id: &str,
        message: video_chat_signaling::Message,
    ) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            for participant in room.participants.values() {
                if participant.id != exclude_participant_id {
                    if let Some(sender) = &participant.sender {
                        let _ = sender.send(message.clone());
                    }
                }
            }
        }
    }

    pub async fn set_peer_connection(
        &self,
        room_id: &str,
        participant_id: &str,
        pc: std::sync::Arc<webrtc::peer_connection::RTCPeerConnection>,
    ) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            if let Some(participant) = room.participants.get(participant_id) {
                if let Ok(mut lock) = participant.peer_connection.lock() {
                    *lock = Some(pc);
                }
            }
        }
    }

    pub async fn get_peer_connection(
        &self,
        room_id: &str,
        participant_id: &str,
    ) -> Option<std::sync::Arc<webrtc::peer_connection::RTCPeerConnection>> {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            if let Some(participant) = room.participants.get(participant_id) {
                if let Ok(lock) = participant.peer_connection.lock() {
                    return lock.clone();
                }
            }
        }
        None
    }

    pub async fn send_message_to_participant(
        &self,
        room_id: &str,
        participant_id: &str,
        message: video_chat_signaling::Message,
    ) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            if let Some(participant) = room.participants.get(participant_id) {
                if let Some(sender) = &participant.sender {
                    let _ = sender.send(message);
                }
            }
        }
    }

    /// Get all participant IDs in a room except the specified one
    pub async fn get_participants_except(
        &self,
        room_id: &str,
        exclude_participant_id: &str,
    ) -> Vec<String> {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            room.participants
                .keys()
                .filter(|id| *id != exclude_participant_id)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all peer connections in a room except the specified participant
    pub async fn get_all_peer_connections_except(
        &self,
        room_id: &str,
        exclude_participant_id: &str,
    ) -> Vec<(String, Arc<RTCPeerConnection>)> {
        let rooms = self.rooms.read().await;
        let mut connections = Vec::new();

        if let Some(room) = rooms.get(room_id) {
            for (pid, participant) in &room.participants {
                if pid != exclude_participant_id {
                    if let Ok(lock) = participant.peer_connection.lock() {
                        if let Some(pc) = lock.as_ref() {
                            connections.push((pid.clone(), pc.clone()));
                        }
                    }
                }
            }
        }
        connections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_manager_creation() {
        let manager = RoomManager::new();
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
            peer_connection: std::sync::Mutex::new(None),
            published_tracks: Vec::new(),
            sender: None,
        };
        assert_eq!(participant.id, "participant-1");
        assert!(participant.peer_connection.lock().unwrap().is_none());
        assert!(participant.published_tracks.is_empty());
    }

    #[tokio::test]
    async fn test_join_room() {
        let manager = RoomManager::new();
        manager
            .join_room("room1".to_string(), "participant1".to_string(), None)
            .await;

        let rooms = manager.rooms.read().await;
        assert!(rooms.contains_key("room1"));
    }

    #[tokio::test]
    async fn test_leave_room() {
        let manager = RoomManager::new();
        manager
            .join_room("room1".to_string(), "participant1".to_string(), None)
            .await;
        manager.leave_room("room1", "participant1").await;

        let rooms = manager.rooms.read().await;
        assert!(!rooms.contains_key("room1"));
    }

    #[tokio::test]
    async fn test_multiple_participants() {
        let manager = RoomManager::new();
        manager
            .join_room("room1".to_string(), "p1".to_string(), None)
            .await;
        manager
            .join_room("room1".to_string(), "p2".to_string(), None)
            .await;
        manager
            .join_room("room1".to_string(), "p3".to_string(), None)
            .await;

        let rooms = manager.rooms.read().await;
        if let Some(room) = rooms.get("room1") {
            assert_eq!(room.participants.len(), 3);
        }
    }

    #[tokio::test]
    async fn test_room_cleanup_when_empty() {
        let manager = RoomManager::new();
        manager
            .join_room("room1".to_string(), "p1".to_string(), None)
            .await;
        manager
            .join_room("room1".to_string(), "p2".to_string(), None)
            .await;
        manager.leave_room("room1", "p1").await;
        manager.leave_room("room1", "p2").await;

        let rooms = manager.rooms.read().await;
        assert!(!rooms.contains_key("room1"));
    }
}
