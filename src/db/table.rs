use crate::Config;
use serde::Deserialize;
use sqlx::{
    postgres::{
        types::Oid,
        PgPool,
    },
    types::Json,
};

#[derive(Clone, Debug, Deserialize)]
pub struct ColumnValue {
    pub name: String,
    pub data_type: String,
    pub position: i32,
    pub nullable: bool,
    pub identity: Option<String>,
    pub generated: Option<String>,
    pub expression: Option<String>,
}

impl ColumnValue {
    pub fn always_generated(&self) -> bool {
        // TODO: Serialize as enums..?
        self.identity.as_deref() == Some("always") ||
        self.identity.as_deref() == Some("stored")
    }
}

pub type Column = Json<ColumnValue>;

#[derive(Clone, Debug, Deserialize)]
pub struct Table {
    pub columns: Vec<Column>,
    pub name: String,
    pub oid: Oid,
    pub schema: String,
}

impl Table {
    pub async fn load(pool: &PgPool, config: &Config, oid: u32) -> Option<Table> {
        sqlx::query_file_as!(
            Table,
            "queries/table-details.sql",
            &config.scope.include,
            &config.scope.exclude,
            Oid(oid)
        )
            .fetch_optional(pool)
            .await
            .unwrap()
    }
}
