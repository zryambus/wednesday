use std::collections::HashMap;

use anyhow::Result;
use sqlx::Row;

pub type Pool = sqlx::PgPool;

pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn init(pool: Pool) -> Result<()> {
        sqlx::migrate!("src/database/sql").run(&pool).await?;

        Ok(())
    }

    pub async fn new(pool: Pool) -> Result<Self> {
        Ok(Self { pool })
    }

    #[tracing::instrument(skip(self))]
    pub async fn is_active(&self, chat_id: i64) -> Result<bool> {
        let row = sqlx::query!(
            r#"SELECT chat_id FROM chats WHERE 'wednesday' = ANY(enabled_notifications)"#,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn add(&self, chat_id: i64) -> Result<()> {
        sqlx::query!(r#"SELECT add_chat($1, 'wednesday')"#, chat_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn remove(&self, chat_id: i64) -> Result<()> {
        sqlx::query!(r#"SELECT remove_chat($1, 'wednesday')"#, chat_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn get_all_active_chats(&self) -> Result<Vec<i64>> {
        let active_chats = sqlx::query!(
            r#"SELECT chat_id FROM chats WHERE 'wednesday' = ANY(enabled_notifications)"#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| row.chat_id)
        .collect();
        Ok(active_chats)
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn is_active_crypto(&self, chat_id: i64) -> Result<bool> {
        let row = sqlx::query!(
            r#"SELECT chat_id FROM chats WHERE 'crypto' = ANY(enabled_notifications)"#,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn add_crypto(&self, chat_id: i64) -> Result<()> {
        sqlx::query!(r#"SELECT add_chat($1, 'crypto')"#, chat_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn remove_crypto(&self, chat_id: i64) -> Result<()> {
        sqlx::query!(r#"SELECT remove_chat($1, 'crypto')"#, chat_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn get_all_active_crypto_chats(&self) -> Result<Vec<i64>> {
        let active_chats =
            sqlx::query(r#"SELECT chat_id FROM chats WHERE 'crypto' = ANY(enabled_notifications)"#)
                .fetch_all(&self.pool)
                .await?
                .iter()
                .map(|row| row.get(0))
                .collect();
        Ok(active_chats)
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_mapping(&self, mapping: HashMap<i64, String>) -> Result<()> {
        for (user_id, username) in mapping {
            sqlx::query!(r#"SELECT update_mapping($1, $2)"#, user_id, username)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_mapping(&self) -> Result<HashMap<i64, String>> {
        let mapping = sqlx::query!(r#"SELECT user_id, username FROM mapping"#)
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|row| (row.user_id, row.username.clone()))
            .collect();
        Ok(mapping)
    }
}
