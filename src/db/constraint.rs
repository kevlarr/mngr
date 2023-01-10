use serde::Deserialize;
use sqlx::{
    postgres::{
        types::{Oid},
        PgPool,
    },
    types::Json,
    Type as SqlType,
};

use super::column::Position;


#[derive(Clone, Debug, Deserialize, PartialEq, SqlType)]
pub enum ForeignKeyMatchType {
    #[serde(rename = "f")]
    Full,
    #[serde(rename = "p")]
    Partial,
    #[serde(rename = "s")]
    Simple,
}

#[derive(Clone, Debug, Deserialize, PartialEq, SqlType)]
pub enum ConstraintType {
    #[serde(rename = "c")]
    Check,
    #[serde(rename = "x")]
    Exclusion,
    #[serde(rename = "f")]
    ForeignKey,
    #[serde(rename = "p")]
    PrimaryKey,
    #[serde(rename = "t")]
    Trigger,
    #[serde(rename = "u")]
    Unique,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ForeignRef {
    #[serde(rename = "confrelid")]
    pub oid: Oid,
    // #[serde(rename = "confkey")]
    // pub columns: Vec<Position>,
    #[serde(rename = "confmatchtype")]
    pub match_type: ForeignKeyMatchType,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CheckConstraint {
    pub name: String,
    pub expression: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ExclusionConstraint {
    pub name: String,
    pub expression: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ForeignKeyConstraint {
    pub name: String,
    pub expression: String,
    pub foreign_ref: ForeignRef,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PrimaryKeyConstraint {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct UniqueConstraint {
    pub name: String,
}


#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ConstraintMap {
    pub check: Option<Vec<CheckConstraint>>,
    pub exclusion: Option<Vec<ExclusionConstraint>>,
    pub foreign_key: Option<Vec<ForeignKeyConstraint>>,
    pub primary_key: Option<Vec<PrimaryKeyConstraint>>,
    pub uniqueness: Option<Vec<UniqueConstraint>>,
}

impl ConstraintMap {
    pub fn requires_unique(&self) -> bool {
        if let Some(pk) = &self.primary_key {
            if !pk.is_empty() {
                return true;
            }
        }

        if let Some(u) = &self.uniqueness {
            if !u.is_empty() {
                return true;
            }
        }

        false
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ConstraintSet {
    pub columns: Vec<Position>,
    // TODO: Get away from using `Json` somehow
    pub constraint_map: Json<ConstraintMap>,
}


impl ConstraintSet {
    pub async fn load(pool: &PgPool, oid: u32) -> Vec<Self> {
        sqlx::query_file_as!(
            Self,
            "queries/table-constraints.sql",
            Oid(oid)
        )
        .fetch_all(pool)
        .await
        .unwrap()
    }
}
