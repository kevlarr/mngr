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
pub struct Constraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub expression: Option<String>,
    // TODO: Get away from using `Json` somehow
    pub foreign_ref: Option<Json<ForeignRef>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ConstraintSet {
    pub columns: Vec<Position>,
    // TODO: Get away from using `Json` somehow
    pub constraints: Vec<Json<Constraint>>,
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


#[derive(Clone, Debug, Deserialize)]
pub struct Constraints {
    pub columns: Vec<Position>,
    pub constraints: Vec<Constraint>,
}

impl Constraints {

    // Because of limitations with using `Option<T>` in nested rows,
    // these are selected as flat structs and then aggregated manually

}
