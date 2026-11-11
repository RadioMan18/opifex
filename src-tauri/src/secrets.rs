//! LLM API keys, stored in the operating system credential store.
//!
//! Keys are only handed to Rust: no command returns one to the webview, and the
//! chat commands read them here. On platforms without a usable credential store
//! the settings table is used as a fallback, so the app still works — it just
//! degrades to the old at-rest behaviour.

use crate::db::DbState;
use rusqlite::params;

const SERVICE: &str = "com.opifex.app";

fn entry(key: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, key).map_err(|e| e.to_string())
}

pub fn set_secret(key: &str, value: &str) -> Result<(), String> {
    entry(key)?.set_password(value).map_err(|e| e.to_string())
}

pub fn get_secret(key: &str) -> Option<String> {
    entry(key).ok()?.get_password().ok()
}

pub fn delete_secret(key: &str) -> Result<(), String> {
    match entry(key)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Moves keys written by older builds out of the plaintext settings table and
/// into the credential store. Rows are only deleted once the copy succeeded, so a
/// machine without a keychain keeps working exactly as before.
pub fn migrate_api_keys(db: &DbState) {
    for key in ["openai_api_key", "anthropic_api_key"] {
        let stored: Option<String> = {
            let conn = &*db.conn.lock().unwrap_or_else(|e| e.into_inner());
            let Ok(mut stmt) = conn.prepare("SELECT value FROM settings WHERE key = ?1") else {
                continue;
            };
            stmt.query_row(params![key], |row| row.get::<_, String>(0)).ok()
        };

        if let Some(value) = stored {
            if value.is_empty() {
                continue;
            }
            if set_secret(key, &value).is_ok() {
                let conn = &*db.conn.lock().unwrap_or_else(|e| e.into_inner());
                let _ = conn.execute("DELETE FROM settings WHERE key = ?1", params![key]);
            }
        }
    }
}
