// Platform-agnostic tests for SFU client

#[cfg(test)]
mod unit_tests {
    use crate::reconnection::ReconnectionManager;
    use std::time::Duration;

    #[test]
    fn test_reconnection_manager_creation() {
        let manager = ReconnectionManager::new(5, Duration::from_millis(100));
        assert_eq!(manager.max_retries, 5);
        assert_eq!(manager.retry_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_reconnection_manager_with_different_params() {
        let manager = ReconnectionManager::new(10, Duration::from_secs(1));
        assert_eq!(manager.max_retries, 10);
        assert_eq!(manager.retry_delay, Duration::from_secs(1));
    }
}

// WASM-specific tests (require browser environment)
#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod wasm_tests {
    use super::*;
    use video_chat_signaling::stun_config::StunConfig;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_sfu_client_initialization() {
        let stun = StunConfig {
            urls: vec!["stun:stun.l.google.com:19302".into()],
        };
        let client = SfuClient::new("test_room".into(), &stun).unwrap();
        assert_eq!(client.room_id, "test_room");
    }

    #[wasm_bindgen_test]
    async fn test_subscribe_to_stream() {
        let stun = StunConfig {
            urls: vec!["stun:stun.l.google.com:19302".into()],
        };
        let client = SfuClient::new("test_room".into(), &stun).unwrap();

        // Test subscribe (currently a stub, but should not error)
        let result = client.subscribe_to_stream("participant-1").await;
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_unsubscribe_from_stream() {
        let stun = StunConfig {
            urls: vec!["stun:stun.l.google.com:19302".into()],
        };
        let client = SfuClient::new("test_room".into(), &stun).unwrap();

        // Test unsubscribe (currently a stub, but should not error)
        let result = client.unsubscribe_from_stream("participant-1").await;
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_multiple_subscriptions() {
        let stun = StunConfig {
            urls: vec!["stun:stun.l.google.com:19302".into()],
        };
        let client = SfuClient::new("test_room".into(), &stun).unwrap();

        // Subscribe to multiple participants
        assert!(client.subscribe_to_stream("p1").await.is_ok());
        assert!(client.subscribe_to_stream("p2").await.is_ok());
        assert!(client.subscribe_to_stream("p3").await.is_ok());

        // Unsubscribe from all
        assert!(client.unsubscribe_from_stream("p1").await.is_ok());
        assert!(client.unsubscribe_from_stream("p2").await.is_ok());
        assert!(client.unsubscribe_from_stream("p3").await.is_ok());
    }
}
