use anyhow::{Result, Context};
use bb8_postgres::tokio_postgres::NoTls;
use rust_embed::RustEmbed;
use sea_query::{Expr, Func, Order, PostgresQueryBuilder, Query};
use std::collections::BTreeMap;
use std::convert::TryInto;

use super::table_active_chats::{ActiveChats, ActiveChatsActions, ActiveCryptoChats};
use super::table_mapping::Mapping;
use super::table_statistics::{Statistics, UpdateKind};
use crate::database::sql_functions::UpdateStatistics;

pub type Connection<'a> = bb8::PooledConnection<'a, bb8_postgres::PostgresConnectionManager<NoTls>>;
pub type Pool = bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>;

pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn init(pool: Pool) -> Result<()> {
        let connection = pool.get().await?;

        let query = ActiveChats::build_create_table_query();
        connection.execute(query.as_str(), &[]).await?;

        let query = ActiveCryptoChats::build_create_table_query();
        connection.execute(query.as_str(), &[]).await?;

        let query = Statistics::build_create_table_query();
        connection.execute(query.as_str(), &[]).await?;

        let query = Mapping::build_create_table_query();
        connection.execute(query.as_str(), &[]).await?;

        Ok(())
    }

    pub async fn new(pool: Pool) -> Result<Self> {
        Ok(Self { pool })
    }

    #[cfg(not(test))]
    async fn connection(&self) -> Result<Connection<'_>> {
        Ok(self.pool.get().await?)
    }

    #[cfg(test)]
    pub(crate) async fn connection(&self) -> Result<Connection<'_>> {
        Ok(self.pool.get().await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn is_active(&self, chat_id: i64) -> Result<bool> {
        let query = ActiveChats::build_is_active_chat_query(chat_id);
        tracing::Span::current().record("query", &query.as_str());
        let active: bool = self
            .connection()
            .await?
            .query_one(query.as_str(), &[])
            .await?
            .get(0);
        Ok(active)
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn add(&self, chat_id: i64) -> Result<()> {
        let query = ActiveChats::build_add_chat_query(chat_id);
        tracing::Span::current().record("query", &query.as_str());
        self.connection()
            .await?
            .execute(query.as_str(), &[])
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn remove(&self, chat_id: i64) -> Result<()> {
        let query = ActiveChats::build_remove_active_chat_query(chat_id);
        tracing::Span::current().record("query", &query.as_str());
        self.connection()
            .await?
            .execute(query.as_str(), &[])
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn get_all_active_chats(&self) -> Result<Vec<i64>> {
        let query = ActiveChats::build_get_active_chats();
        tracing::Span::current().record("query", &query.as_str());
        let stmt = self.connection().await?.prepare(query.as_str()).await?;
        let active_chats: Vec<i64> = self
            .connection()
            .await?
            .query(&stmt, &[])
            .await?
            .iter()
            .map(|row| row.get(0))
            .collect();
        Ok(active_chats)
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn is_active_crypto(&self, chat_id: i64) -> Result<bool> {
        let query = ActiveCryptoChats::build_is_active_chat_query(chat_id);
        tracing::Span::current().record("query", &query.as_str());
        let active: bool = self
            .connection()
            .await?
            .query_one(query.as_str(), &[])
            .await?
            .get(0);
        Ok(active)
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn add_crypto(&self, chat_id: i64) -> Result<()> {
        let query = ActiveCryptoChats::build_add_chat_query(chat_id);
        tracing::Span::current().record("query", &query.as_str());
        self.connection()
            .await?
            .execute(query.as_str(), &[])
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn remove_crypto(&self, chat_id: i64) -> Result<()> {
        let query = ActiveCryptoChats::build_remove_active_chat_query(chat_id);
        tracing::Span::current().record("query", &query.as_str());
        self.connection()
            .await?
            .execute(query.as_str(), &[])
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn get_all_active_crypto_chats(&self) -> Result<Vec<i64>> {
        let query = ActiveCryptoChats::build_get_active_chats();
        tracing::Span::current().record("query", &query.as_str());
        let stmt = self.connection().await?.prepare(query.as_str()).await?;
        let active_chats: Vec<i64> = self
            .connection()
            .await?
            .query(&stmt, &[])
            .await?
            .iter()
            .map(|row| row.get::<usize, i64>(0))
            .collect();
        Ok(active_chats)
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn update_statistics(
        &self,
        chat_id: i64,
        user_id: u64,
        kind: UpdateKind,
    ) -> Result<()> {
        let today = chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::east_opt(3 * 3600)
                .with_context(|| "Could not set timezone for today")?)
            .naive_local();

        let query = Query::select()
            .expr(Func::cust(UpdateStatistics).args([
                chat_id.into(),
                user_id.into(),
                kind.into(),
                today.into(),
            ]))
            .to_string(PostgresQueryBuilder);
        tracing::Span::current().record("query", &query.as_str());

        self.connection()
            .await?
            .simple_query(query.as_str())
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(query))]
    pub async fn get_statistics(
        &self,
        chat_id: i64,
        user_id: u64,
    ) -> Result<BTreeMap<UpdateKind, usize>> {
        let mut result = BTreeMap::new();
        result.insert(UpdateKind::TextMessage, 0);
        result.insert(UpdateKind::Sticker, 0);
        result.insert(UpdateKind::ForwardedMeme, 0);

        let today = chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::east_opt(3 * 3600)
                .with_context(|| "Failed to set timezone")?)
            .naive_local();

        let query = Query::select()
            .from(Statistics::Table)
            .columns(vec![Statistics::Kind, Statistics::Count])
            .and_where(Expr::col(Statistics::Chat).eq(chat_id))
            .and_where(Expr::col(Statistics::User).eq(user_id))
            .and_where(Expr::col(Statistics::Date).eq(today))
            .order_by(Statistics::Kind, Order::Asc)
            .to_string(PostgresQueryBuilder);
        tracing::Span::current().record("query", &query.as_str());

        let data: Vec<(UpdateKind, usize)> = self
            .connection()
            .await?
            .query(query.as_str(), &[])
            .await?
            .iter()
            .map(|row| {
                let kind_i32: i32 = row.get(0);
                let count: i64 = row.get(1);
                let kind: UpdateKind = kind_i32.try_into().unwrap();
                (kind, count as usize)
            })
            .collect();

        for (key, value) in data {
            result.insert(key, value);
        }

        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_mapping(&self, mapping: Vec<(u64, String)>) -> Result<()> {
        for (user_id, username) in mapping {
            let query = Mapping::build_set_query(user_id, username)?;
            self.connection()
                .await?
                .simple_query(query.as_str())
                .await?;
        }
        Ok(())
    }
}

pub(crate) trait CreateTable {
    fn build_create_table_query() -> String;
}

#[derive(RustEmbed)]
#[folder = "src/database/sql"]
pub struct SQLInit;
