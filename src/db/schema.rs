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
pub struct SchemaTableValue {
    pub name: String,
    pub oid: Oid,
}

pub type SchemaTable = Json<SchemaTableValue>;

#[derive(Clone, Debug)]
pub struct Schema {
    pub name: String,
    pub tables: Vec<SchemaTable>,
}

#[derive(Clone, Debug)]
pub struct Schemas(Vec<Schema>);

impl Schemas {
    pub async fn load(pool: &PgPool, config: &Config) -> Schemas {
        let schemas = sqlx::query_file_as!(
            Schema,
            "queries/tables-by-schema.sql",
            &config.scope.include,
            &config.scope.exclude
        )
            .fetch_all(pool)
            .await
            .unwrap();

        Schemas(schemas)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Schema> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
