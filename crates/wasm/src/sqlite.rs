use video_chat_core::error::Result;

#[allow(dead_code)]
pub struct Database {
    // conn: Connection, // Removed until correct WASM SQLite crate is identified
}

#[allow(dead_code)]
impl Database {
    pub fn open_in_memory() -> Result<Self> {
        log::warn!("SQLite-WASM not yet fully integrated. Using mock database.");
        Ok(Self {})
    }

    pub fn insert_message(
        &self,
        _id: &str,
        _room_id: &str,
        _sender_id: &str,
        _content: &str,
        _timestamp: i64,
    ) -> Result<()> {
        log::info!("Mock DB: insert_message not persisted");
        Ok(())
    }

    pub fn get_messages(&self, _room_id: &str) -> Result<Vec<(String, String, String, i64)>> {
        log::info!("Mock DB: get_messages returning empty");
        Ok(Vec::new())
    }
}
