#[cfg(test)]
mod tests {
    use crate::message::DataChannelMessage;
    use crate::transient_storage::TransientStorage;
    use serde_json;

    #[test]
    fn test_message_serialization() {
        let msg = DataChannelMessage::new_chat("user1".into(), "hello".into());
        let json = serde_json::to_string(&msg).unwrap();
        
        // deserialize
        let deserialized: DataChannelMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
        
        if let DataChannelMessage::Chat { sender_id, content, .. } = deserialized {
            assert_eq!(sender_id, "user1");
            assert_eq!(content, "hello");
        } else {
            panic!("Wrong message type");
        }
    }

    #[test]
    fn test_transient_storage() {
        let mut storage = TransientStorage::new(2);
        
        let msg1 = DataChannelMessage::new_chat("u1".into(), "1".into());
        let msg2 = DataChannelMessage::new_chat("u1".into(), "2".into());
        let msg3 = DataChannelMessage::new_chat("u1".into(), "3".into());

        storage.add_message(msg1.clone());
        assert_eq!(storage.get_messages().len(), 1);

        storage.add_message(msg2.clone());
        assert_eq!(storage.get_messages().len(), 2);

        // Should drop msg1
        storage.add_message(msg3.clone());
        let messages = storage.get_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], msg2);
        assert_eq!(messages[1], msg3);
    }
}
