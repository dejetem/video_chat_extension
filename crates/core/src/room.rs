use crate::error::Result;
use crate::participant::Participant;
use std::collections::HashMap;

pub struct Room {
    pub id: String,
    pub participants: HashMap<String, Participant>,
}

impl Room {
    pub fn new(id: String) -> Self {
        Self {
            id,
            participants: HashMap::new(),
        }
    }

    pub fn add_participant(&mut self, participant: Participant) -> Result<()> {
        self.participants
            .insert(participant.id.clone(), participant);
        Ok(())
    }

    pub fn remove_participant(&mut self, participant_id: &str) -> Result<Option<Participant>> {
        Ok(self.participants.remove(participant_id))
    }

    pub fn get_participant(&self, participant_id: &str) -> Option<&Participant> {
        self.participants.get(participant_id)
    }

    pub fn list_participants(&self) -> Vec<&Participant> {
        self.participants.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_management() {
        let mut room = Room::new("test-room".into());
        let p1 = Participant::new("p1".into(), "Alice".into(), true);
        let p2 = Participant::new("p2".into(), "Bob".into(), false);

        room.add_participant(p1).unwrap();
        room.add_participant(p2).unwrap();

        assert_eq!(room.participants.len(), 2);
        assert!(room.get_participant("p1").is_some());
        assert_eq!(room.get_participant("p1").unwrap().name, "Alice");

        let removed = room.remove_participant("p2").unwrap();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Bob");
        assert_eq!(room.participants.len(), 1);

        let participants = room.list_participants();
        assert_eq!(participants.len(), 1);
    }
}
