use serde::Deserialize;
use sqlx::{
    postgres::{
        types::Oid,
        PgPool,
    },
    FromRow,
    Type as SqlType,
};

pub type Position = i16;

#[derive(Clone, Debug, Deserialize, PartialEq, SqlType)]
pub enum Identity {
    #[sqlx(rename = "a")]
    AlwaysGenerated,
    #[sqlx(rename = "g")]
    GeneratedByDefault,
}

#[derive(Clone, Debug, Deserialize, PartialEq, SqlType)]
pub enum Generated {
    #[sqlx(rename = "s")]
    Stored,
}


#[derive(Clone, Debug, FromRow)]
pub struct Column {
    pub data_type: String,
    pub description: Option<String>,
    pub expression: Option<String>,
    pub generated: Option<Generated>,
    pub identity: Option<Identity>,
    pub name: String,
    pub nullable: bool,
    pub position: Position,
}

impl Column {
    pub async fn load(pool: &PgPool, oid: u32) -> Vec<Self> {
        sqlx::query_file_as!(
            Self,
            "queries/table-columns.sql",
            Oid(oid),
        )
            .fetch_all(pool)
            .await
            .unwrap()
    }

    pub fn always_generated(&self) -> bool {
        self.identity == Some(Identity::AlwaysGenerated) ||
        self.generated == Some(Generated::Stored)
    }

    pub fn required(&self) -> bool {
        !self.nullable && self.expression.is_none()
    }
}
