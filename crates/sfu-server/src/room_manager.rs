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
