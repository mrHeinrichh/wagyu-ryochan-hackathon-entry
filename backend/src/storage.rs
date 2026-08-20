//! Durable SQLite storage for replayable receipts and watchlist state.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, params};
use tracing::warn;

use crate::domain::{DecisionReceipt, WatchItem};

const SCHEMA: &str = r#"
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
CREATE TABLE IF NOT EXISTS receipts (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL UNIQUE,
    symbol TEXT NOT NULL,
    created_at TEXT NOT NULL,
    payload TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS receipts_created_at_idx ON receipts(created_at DESC);
CREATE TABLE IF NOT EXISTS watchlist (
    symbol TEXT PRIMARY KEY,
    updated_at TEXT NOT NULL,
    payload TEXT NOT NULL
);
"#;

/// Opens short-lived connections per operation so handlers never share a
/// connection across threads. SQLite remains the durable source of truth.
#[derive(Debug, Clone)]
pub(crate) struct Persistence {
    path: PathBuf,
}

impl Persistence {
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let storage = Self { path };
        storage
            .connection()?
            .execute_batch(SCHEMA)
            .map_err(|error| error.to_string())?;
        Ok(storage)
    }

    pub(crate) fn save_receipt(&self, receipt: &DecisionReceipt) -> Result<(), String> {
        let payload = serde_json::to_string(receipt).map_err(|error| error.to_string())?;
        let connection = self.connection()?;
        connection.execute(
            "INSERT OR REPLACE INTO receipts (id, run_id, symbol, created_at, payload) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![receipt.id, receipt.run_id, receipt.symbol, receipt.created_at, payload],
        ).map_err(|error| error.to_string())?;
        connection.execute(
            "DELETE FROM receipts WHERE id NOT IN (SELECT id FROM receipts ORDER BY created_at DESC LIMIT 100)",
            [],
        ).map_err(|error| error.to_string())?;
        Ok(())
    }

    pub(crate) fn load_receipts(&self, limit: usize) -> Result<Vec<DecisionReceipt>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare("SELECT payload FROM receipts ORDER BY created_at DESC LIMIT ?1")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([limit as i64], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        let mut receipts = Vec::new();
        for row in rows {
            let payload = row.map_err(|error| error.to_string())?;
            match serde_json::from_str(&payload) {
                Ok(receipt) => receipts.push(receipt),
                Err(error) => warn!("skipping unreadable persisted receipt: {error}"),
            }
        }
        Ok(receipts)
    }

    pub(crate) fn save_watch_item(&self, item: &WatchItem) -> Result<(), String> {
        let payload = serde_json::to_string(item).map_err(|error| error.to_string())?;
        self.connection()?.execute(
            "INSERT OR REPLACE INTO watchlist (symbol, updated_at, payload) VALUES (?1, ?2, ?3)",
            params![item.symbol, chrono::Utc::now().to_rfc3339(), payload],
        ).map_err(|error| error.to_string())?;
        Ok(())
    }

    pub(crate) fn delete_watch_item(&self, symbol: &str) -> Result<(), String> {
        self.connection()?
            .execute("DELETE FROM watchlist WHERE symbol = ?1", [symbol])
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub(crate) fn load_watchlist(&self) -> Result<HashMap<String, WatchItem>, String> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare("SELECT payload FROM watchlist ORDER BY symbol")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        let mut items = Vec::new();
        for row in rows {
            let payload = row.map_err(|error| error.to_string())?;
            match serde_json::from_str::<WatchItem>(&payload) {
                Ok(item) => items.push(item),
                Err(error) => warn!("skipping unreadable persisted watch item: {error}"),
            }
        }
        Ok(items
            .into_iter()
            .map(|item| (item.symbol.clone(), item))
            .collect())
    }

    fn connection(&self) -> Result<Connection, String> {
        let connection = Connection::open(&self.path).map_err(|error| error.to_string())?;
        connection
            .busy_timeout(std::time::Duration::from_secs(3))
            .map_err(|error| error.to_string())?;
        Ok(connection)
    }
}
