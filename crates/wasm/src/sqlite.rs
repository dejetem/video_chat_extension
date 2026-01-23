use crate::schema::SCHEMA_V1;
use sqlite_wasm_rs::export::Connection;
use video_chat_core::error::Result;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| video_chat_core::error::Error::Internal(e.to_string()))?;

        conn.execute(SCHEMA_V1)
            .map_err(|e| video_chat_core::error::Error::Internal(e.to_string()))?;

        Ok(Self { conn })
    }

    pub fn open_persistent(db_name: &str) -> Result<Self> {
        // In a real WASM environment with sqlite-wasm-rs,
        // this might use OPFS. For now, we use open() which
        // depending on the build might be persistent or memory-backed.
        let conn = Connection::open(db_name)
            .map_err(|e| video_chat_core::error::Error::Internal(e.to_string()))?;

        conn.execute(SCHEMA_V1)
            .map_err(|e| video_chat_core::error::Error::Internal(e.to_string()))?;

        Ok(Self { conn })
    }

    pub fn insert_message(
        &self,
        id: &str,
        room_id: &str,
        sender_id: &str,
        content: &str,
        timestamp: i64,
    ) -> Result<()> {
        let sql = "INSERT INTO chat_logs (id, room_id, sender_id, content, timestamp) VALUES (?, \
                   ?, ?, ?, ?)";
        self.conn
            .execute_with_params(
                sql,
                &[
                    id.into(),
                    room_id.into(),
                    sender_id.into(),
                    content.into(),
                    timestamp.into(),
                ],
            )
            .map_err(|e| video_chat_core::error::Error::Internal(e.to_string()))?;

        Ok(())
    }

    pub fn get_messages(&self, room_id: &str) -> Result<Vec<(String, String, String, i64)>> {
        let sql = "SELECT sender_id, content, id, timestamp FROM chat_logs WHERE room_id = ? \
                   ORDER BY timestamp ASC";
        let rows = self
            .conn
            .query_with_params(sql, &[room_id.into()])
            .map_err(|e| video_chat_core::error::Error::Internal(e.to_string()))?;

        let mut messages = Vec::new();
        for row in rows {
            // Mapping logic based on sqlite-wasm-rs row access
            let sender_id = row.get(0).unwrap_or_default();
            let content = row.get(1).unwrap_or_default();
            let id = row.get(2).unwrap_or_default();
            let timestamp: i64 = row.get(3).unwrap_or_default();
            messages.push((sender_id, content, id, timestamp));
        }

        Ok(messages)
    }
}
