use sea_query::{Iden, Table, ColumnDef, PostgresQueryBuilder, Query, Expr, Func};
use crate::database::sql_functions::Exists;

#[derive(Iden)]
pub enum ActiveChats {
    Table,
    ChatId
}

#[derive(Iden)]
pub enum ActiveCryptoChats {
    Table,
    ChatId
}

pub trait ActiveT {
    fn table() -> Self;
    fn field() -> Self;
}

impl ActiveT for ActiveChats {
    fn table() -> Self {
        Self::Table
    }

    fn field() -> Self {
        Self::ChatId
    }
}

impl ActiveT for ActiveCryptoChats {
    fn table() -> Self {
        Self::Table
    }

    fn field() -> Self {
        Self::ChatId
    }
}

trait CreateTableForActiveChats<T> where T: ActiveT + Iden + 'static {
    fn build_create_table_query_impl() -> String {
        Table::create()
            .if_not_exists()
            .table(T::table())
            .col(ColumnDef::new(T::field()).big_integer().unique_key().primary_key().not_null())
            .build(PostgresQueryBuilder)
    }
}

impl CreateTableForActiveChats<Self> for ActiveChats{}
impl CreateTableForActiveChats<Self> for ActiveCryptoChats{}

impl super::db::CreateTable for ActiveChats {
    fn build_create_table_query() -> String {
        ActiveChats::build_create_table_query_impl()
    }
}

impl super::db::CreateTable for ActiveCryptoChats {
    fn build_create_table_query() -> String {
        ActiveCryptoChats::build_create_table_query_impl()
    }
}

pub trait ActiveChatsActions<T: ActiveT + Iden + 'static> {
    fn build_is_active_chat_query(chat_id: i64) -> String {
        let query = Query::select()
            .expr(Expr::val(1))
            .from(T::table())
            .and_where(Expr::col(T::field()).eq(chat_id))
            .to_string(PostgresQueryBuilder);

        Query::select()
            .expr(Func::cust(Exists).arg(Expr::cust(&query)))
            .limit(1)
            .to_string(PostgresQueryBuilder)
    }

    fn build_add_chat_query(chat_id: i64) -> String {
        Query::insert()
            .into_table(T::table())
            .columns(vec![
                T::field()
            ])
            .values(vec![
                chat_id.into()
            ])
            .unwrap()
            .to_string(PostgresQueryBuilder)
    }

    fn build_remove_active_chat_query(chat_id: i64) -> String {
        Query::delete()
            .from_table(T::table())
            .and_where(Expr::col(T::field()).eq(chat_id))
            .to_string(PostgresQueryBuilder)
    }

    fn build_get_active_chats() -> String {
        Query::select()
            .from(T::table())
            .column(T::field())
            .to_string(PostgresQueryBuilder)
    }
}

impl ActiveChatsActions<Self> for ActiveChats {}
impl ActiveChatsActions<Self> for ActiveCryptoChats {}