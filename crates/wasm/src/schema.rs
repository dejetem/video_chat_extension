// Database schema definition
// Currently defined directly in sqlite.rs for simplicity, but can be moved here for migrations
#[allow(dead_code)]
pub const SCHEMA_V1: &str = "
    CREATE TABLE IF NOT EXISTS chat_logs (
        id TEXT PRIMARY KEY,
        room_id TEXT NOT NULL,
        sender_id TEXT NOT NULL,
        content TEXT NOT NULL,
        timestamp INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_room_id ON chat_logs(room_id);
    CREATE INDEX IF NOT EXISTS idx_timestamp ON chat_logs(timestamp);
";
