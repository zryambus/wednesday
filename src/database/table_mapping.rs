use crate::database::sql_functions::UpdateMapping;
use sea_query::{ColumnDef, Expr, Func, Iden, PostgresQueryBuilder, Query, Table};

#[derive(Iden)]
pub enum Mapping {
    Table,
    UserId,
    Username,
}

impl super::db::CreateTable for Mapping {
    fn build_create_table_query() -> String {
        Table::create()
            .if_not_exists()
            .table(Self::Table)
            .col(
                ColumnDef::new(Self::UserId)
                    .big_integer()
                    .unique_key()
                    .not_null(),
            )
            .col(ColumnDef::new(Self::Username).string().not_null())
            .build(PostgresQueryBuilder)
    }
}

impl Mapping {
    pub fn build_set_query(user_id: i64, username: String) -> anyhow::Result<String> {
        Ok(Query::select()
            .expr(Func::cust(UpdateMapping).args(vec![Expr::val(user_id), Expr::val(username)]))
            .to_string(PostgresQueryBuilder))
    }
}
