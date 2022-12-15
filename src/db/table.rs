use crate::Config;
use serde::Deserialize;
use sqlx::{
    postgres::{
        types::Oid,
        PgPool,
    },
    types::Json,
};

pub type Position = i32;

#[derive(Clone, Debug, Deserialize)]
pub struct ColumnValue {
    pub data_type: String,
    pub expression: Option<String>,
    pub generated: Option<String>,
    pub identity: Option<String>,
    pub name: String,
    pub nullable: bool,
    pub position: Position,
}

impl ColumnValue {
    pub fn always_generated(&self) -> bool {
        // TODO: Serialize as enums..?
        self.identity.as_deref() == Some("always") ||
        self.identity.as_deref() == Some("stored")
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ForeignRef {
    pub oid: Oid,
    pub columns: Vec<Position>,
    // TODO: This as well?
    // when 'f' then 'full'
    // when 'p' then 'partial'
    // when 's' then 'simple'
    pub match_type: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConstraintValue {
    pub name: String,
    pub columns: Vec<Position>,
    // TODO: Can this be deserialized to an enum?
    // when 'c' then 'check'
    // when 'f' then 'foreign_key'
    // when 'p' then 'primary_key'
    // when 'u' then 'unique'
    // when 't' then 'constraint_trigger'
    // when 'x' then 'exclusion'
    pub constraint_type: String,
    pub expression: Option<String>,
    pub foreign_ref: Option<ForeignRef>,
}

pub type Column = Json<ColumnValue>;
pub type Constraint = Json<ConstraintValue>;

#[derive(Clone, Debug, Deserialize)]
pub struct Table {
    pub columns: Vec<Column>,
    pub constraints: Vec<Constraint>,
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
