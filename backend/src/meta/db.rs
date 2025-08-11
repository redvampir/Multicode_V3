use crate::meta::VisualMeta;
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
    Ok(())
}

/// Insert or replace a [`VisualMeta`] entry in the database.
pub async fn upsert(pool: &SqlitePool, meta: &VisualMeta) -> Result<(), sqlx::Error> {
    let json = serde_json::to_string(meta).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    sqlx::query("INSERT OR REPLACE INTO visual_meta (id, meta) VALUES (?, ?)")
        .bind(&meta.id)
        .bind(json)
        .execute(pool)
        .await?;
    Ok(())
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
