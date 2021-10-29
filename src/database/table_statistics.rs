use sea_query::{ColumnDef, Iden, PostgresQueryBuilder, Table};
use std::convert::TryFrom;

#[derive(Iden)]
pub enum Statistics {
    Table,
    Chat,
    User,
    Count,
    Date,
    Kind,
}

#[repr(i64)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub enum UpdateKind {
    TextMessage = 0,
    Sticker = 1,
    ForwardedMeme = 2,
}

impl TryFrom<i32> for UpdateKind {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == UpdateKind::TextMessage as i32 => Ok(UpdateKind::TextMessage),
            x if x == UpdateKind::Sticker as i32 => Ok(UpdateKind::Sticker),
            x if x == UpdateKind::ForwardedMeme as i32 => Ok(UpdateKind::ForwardedMeme),
            _ => Err(()),
        }
    }
}

impl Into<sea_query::Value> for UpdateKind {
    fn into(self) -> sea_query::Value {
        sea_query::Value::Int(Some(self as i32))
    }
}

impl super::db::CreateTable for Statistics {
    fn build_create_table_query() -> String {
        Table::create()
            .table(Statistics::Table)
            .if_not_exists()
            .col(ColumnDef::new(Statistics::Chat).big_integer().not_null())
            .col(ColumnDef::new(Statistics::User).big_integer().not_null())
            .col(ColumnDef::new(Statistics::Count).big_integer().default(0))
            .col(ColumnDef::new(Statistics::Date).date().not_null())
            .col(ColumnDef::new(Statistics::Kind).integer().not_null())
            .build(PostgresQueryBuilder)
    }
}

impl Statistics {}
