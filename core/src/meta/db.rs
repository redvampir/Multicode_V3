use crate::meta::VisualMeta;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// Initialize the database by creating the `visual_meta` table if it does not exist.
pub async fn init(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS visual_meta (
            id TEXT PRIMARY KEY,
            meta TEXT NOT NULL
        )"#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS meta_history (
            meta_id TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            payload TEXT NOT NULL
        )"#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Insert or replace a [`VisualMeta`] entry in the database.
pub async fn upsert(pool: &SqlitePool, meta: &VisualMeta) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    if let Some(row) = sqlx::query("SELECT meta FROM visual_meta WHERE id = ?")
        .bind(&meta.id)
        .fetch_optional(&mut *tx)
        .await?
    {
        let json: String = row.try_get("meta")?;
        sqlx::query("INSERT INTO meta_history (meta_id, timestamp, payload) VALUES (?, ?, ?)")
            .bind(&meta.id)
            .bind(Utc::now().to_rfc3339())
            .bind(json)
            .execute(&mut *tx)
            .await?;
    }

    let json = serde_json::to_string(meta).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    sqlx::query("INSERT OR REPLACE INTO visual_meta (id, meta) VALUES (?, ?)")
        .bind(&meta.id)
        .bind(json)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub payload: VisualMeta,
}

pub async fn history(pool: &SqlitePool, id: &str) -> Result<Vec<HistoryEntry>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT timestamp, payload FROM meta_history WHERE meta_id = ? ORDER BY timestamp DESC",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    let mut out = Vec::new();
    for row in rows {
        let ts: String = row.try_get("timestamp")?;
        let payload: String = row.try_get("payload")?;
        let timestamp = ts
            .parse::<DateTime<Utc>>()
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        let payload =
            serde_json::from_str(&payload).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        out.push(HistoryEntry { timestamp, payload });
    }
    Ok(out)
}

pub async fn rollback(
    pool: &SqlitePool,
    id: &str,
    timestamp: &str,
) -> Result<VisualMeta, sqlx::Error> {
    let mut tx = pool.begin().await?;
    if let Some(row) = sqlx::query("SELECT meta FROM visual_meta WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
    {
        let current: String = row.try_get("meta")?;
        sqlx::query("INSERT INTO meta_history (meta_id, timestamp, payload) VALUES (?, ?, ?)")
            .bind(id)
            .bind(Utc::now().to_rfc3339())
            .bind(current)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(row) =
        sqlx::query("SELECT payload FROM meta_history WHERE meta_id = ? AND timestamp = ?")
            .bind(id)
            .bind(timestamp)
            .fetch_optional(&mut *tx)
            .await?
    {
        let payload: String = row.try_get("payload")?;
        sqlx::query("INSERT OR REPLACE INTO visual_meta (id, meta) VALUES (?, ?)")
            .bind(id)
            .bind(&payload)
            .execute(&mut *tx)
            .await?;
        let meta = serde_json::from_str(&payload).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        tx.commit().await?;
        Ok(meta)
    } else {
        tx.rollback().await?;
        Err(sqlx::Error::RowNotFound)
    }
}

/// Fetch a [`VisualMeta`] by id.
pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<VisualMeta>, sqlx::Error> {
    if let Some(row) = sqlx::query("SELECT meta FROM visual_meta WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
    {
        let json: String = row.try_get("meta")?;
        let meta = serde_json::from_str(&json).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        Ok(Some(meta))
    } else {
        Ok(None)
    }
}

/// Delete a metadata entry by id.
pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM visual_meta WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// List all metadata entries in the database.
pub async fn list(pool: &SqlitePool) -> Result<Vec<VisualMeta>, sqlx::Error> {
    let rows = sqlx::query("SELECT meta FROM visual_meta")
        .fetch_all(pool)
        .await?;
    let mut metas = Vec::new();
    for row in rows {
        let json: String = row.try_get("meta")?;
        let meta = serde_json::from_str(&json).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        metas.push(meta);
    }
    Ok(metas)
}
