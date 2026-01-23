use video_chat_core::error::Result;

#[cfg(not(target_arch = "wasm32"))]
pub struct Database;

#[cfg(not(target_arch = "wasm32"))]
impl Database {
    pub fn open_in_memory() -> Result<Self> {
        log::warn!("SQLite-WASM not supported on this platform. Using mock.");
        Ok(Self)
    }

    pub fn open_persistent(_db_name: &str) -> Result<Self> {
        log::warn!("SQLite-WASM not supported on this platform. Using mock.");
        Ok(Self)
    }

    pub fn insert_message(
        &self,
        _id: &str,
        _room_id: &str,
        _sender_id: &str,
        _content: &str,
        _timestamp: i64,
    ) -> Result<()> {
        Ok(())
    }

    pub fn get_messages(&self, _room_id: &str) -> Result<Vec<(String, String, String, i64)>> {
        Ok(Vec::new())
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use super::*;
    use crate::schema::SCHEMA_V1;
    use sqlite_wasm_rs::export::*;
    use std::ffi::{CStr, CString};
    use std::ptr;

    pub struct Database {
        pub(crate) db: *mut sqlite3,
    }

    unsafe impl Send for Database {}
    unsafe impl Sync for Database {}

    impl Database {
        pub fn open_in_memory() -> Result<Self> {
            let mut db = ptr::null_mut();
            let filename = CString::new(":memory:").unwrap();
            unsafe {
                let rc = sqlite3_open_v2(
                    filename.as_ptr(),
                    &mut db,
                    SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
                    ptr::null(),
                );
                if rc != SQLITE_OK {
                    return Err(video_chat_core::error::Error::Internal(format!(
                        "Failed to open in-memory DB: {}",
                        rc
                    )));
                }
            }
            let db = Database { db };
            db.execute(SCHEMA_V1)?;
            Ok(db)
        }

        pub fn open_persistent(db_name: &str) -> Result<Self> {
            let mut db = ptr::null_mut();
            let filename = CString::new(db_name).unwrap();
            unsafe {
                let rc = sqlite3_open_v2(
                    filename.as_ptr(),
                    &mut db,
                    SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
                    ptr::null(),
                );
                if rc != SQLITE_OK {
                    return Err(video_chat_core::error::Error::Internal(format!(
                        "Failed to open persistent DB {}: {}",
                        db_name, rc
                    )));
                }
            }
            let db = Database { db };
            db.execute(SCHEMA_V1)?;
            Ok(db)
        }

        pub fn execute(&self, sql: &str) -> Result<()> {
            let sql_c = CString::new(sql).unwrap();
            unsafe {
                let mut errmsg = ptr::null_mut();
                let rc = sqlite3_exec(self.db, sql_c.as_ptr(), None, ptr::null_mut(), &mut errmsg);
                if rc != SQLITE_OK {
                    let msg = if errmsg.is_null() {
                        "Unknown error".to_string()
                    } else {
                        let s = CStr::from_ptr(errmsg).to_string_lossy().into_owned();
                        sqlite3_free(errmsg.cast());
                        s
                    };
                    return Err(video_chat_core::error::Error::Internal(format!(
                        "SQL error: {}",
                        msg
                    )));
                }
            }
            Ok(())
        }

        pub fn insert_message(
            &self,
            id: &str,
            room_id: &str,
            sender_id: &str,
            content: &str,
            timestamp: i64,
        ) -> Result<()> {
            let sql = "INSERT INTO chat_logs (id, room_id, sender_id, content, timestamp) VALUES \
                       (?, ?, ?, ?, ?)";
            let sql_c = CString::new(sql).unwrap();
            unsafe {
                let mut stmt = ptr::null_mut();
                let rc =
                    sqlite3_prepare_v2(self.db, sql_c.as_ptr(), -1, &mut stmt, ptr::null_mut());
                if rc != SQLITE_OK {
                    return Err(video_chat_core::error::Error::Internal(
                        "Prepare failed".to_string(),
                    ));
                }
                sqlite3_bind_text(stmt, 1, CString::new(id).unwrap().as_ptr(), -1, None);
                sqlite3_bind_text(stmt, 2, CString::new(room_id).unwrap().as_ptr(), -1, None);
                sqlite3_bind_text(stmt, 3, CString::new(sender_id).unwrap().as_ptr(), -1, None);
                sqlite3_bind_text(stmt, 4, CString::new(content).unwrap().as_ptr(), -1, None);
                sqlite3_bind_int64(stmt, 5, timestamp);
                sqlite3_step(stmt);
                sqlite3_finalize(stmt);
            }
            Ok(())
        }

        pub fn get_messages(&self, room_id: &str) -> Result<Vec<(String, String, String, i64)>> {
            let sql = "SELECT sender_id, content, id, timestamp FROM chat_logs WHERE room_id = ? \
                       ORDER BY timestamp ASC";
            let sql_c = CString::new(sql).unwrap();
            let mut messages = Vec::new();
            unsafe {
                let mut stmt = ptr::null_mut();
                sqlite3_prepare_v2(self.db, sql_c.as_ptr(), -1, &mut stmt, ptr::null_mut());
                sqlite3_bind_text(stmt, 1, CString::new(room_id).unwrap().as_ptr(), -1, None);
                while sqlite3_step(stmt) == SQLITE_ROW {
                    let s_id = CStr::from_ptr(sqlite3_column_text(stmt, 0).cast())
                        .to_string_lossy()
                        .into_owned();
                    let cont = CStr::from_ptr(sqlite3_column_text(stmt, 1).cast())
                        .to_string_lossy()
                        .into_owned();
                    let m_id = CStr::from_ptr(sqlite3_column_text(stmt, 2).cast())
                        .to_string_lossy()
                        .into_owned();
                    let ts = sqlite3_column_int64(stmt, 3);
                    messages.push((s_id, cont, m_id, ts));
                }
                sqlite3_finalize(stmt);
            }
            Ok(messages)
        }
    }

    impl Drop for Database {
        fn drop(&mut self) {
            unsafe {
                sqlite3_close_v2(self.db);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_impl::Database;
