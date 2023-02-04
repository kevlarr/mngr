use serde::Deserialize;
use sqlx::postgres::{
    types::Oid,
    PgPool,
};

use crate::Config;
use super::{Column, ConstraintSet};

#[derive(Clone, Debug, Deserialize)]
pub struct Meta {
    pub oid: Oid,
    pub name: String,
    pub schema: String,
    pub description: Option<String>,
}

impl Meta {
    pub async fn load(pool: &PgPool, config: &Config, oid: u32) -> Option<Self> {
        sqlx::query_file_as!(
            Self,
            "queries/table.sql",
            Oid(oid),
            &config.scope.include,
            &config.scope.exclude,
        )
            .fetch_optional(pool)
            .await
            .unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct Table {
    pub meta: Meta,
    pub columns: Vec<Column>,
    pub constraints: Vec<ConstraintSet>,
}

impl Table {
    pub async fn load(pool: &PgPool, config: &Config, oid: u32) -> Option<Self> {
        let meta = Meta::load(pool, config, oid).await?;
        let columns = Column::load(pool, oid).await;
        let constraints = ConstraintSet::load_all(pool, oid).await;

        Some(Self { meta, columns, constraints })
    }
}
