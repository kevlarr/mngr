use std::collections::HashMap;

use serde::{Deserialize, Deserializer};
use sqlx::{
    postgres::{
        types::Oid,
        PgPool,
    },
    types::Json,
};

use crate::Config;

pub type Position = i32;

#[derive(Clone, Debug)]
pub enum ForeignKeyMatchType {
    Full,
    Partial,
    Simple,
}

impl<'de> Deserialize<'de> for ForeignKeyMatchType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "f" => Self::Full,
            "p" => Self::Partial,
            "s" => Self::Simple,
            t => panic!("Unexpected foreign key match type `{}`", t)
        })
    }
}

#[derive(Clone, Debug)]
pub enum ConstraintType {
    Check,
    Exclusion,
    ForeignKey,
    PrimaryKey,
    Trigger,
    Unique,
}

impl<'de> Deserialize<'de> for ConstraintType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "c" => Self::Check,
            "f" => Self::ForeignKey,
            "p" => Self::PrimaryKey,
            "t" => Self::Trigger,
            "u" => Self::Unique,
            "x" => Self::Exclusion,
            t => panic!("Unexpected constraint type `{}`", t)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IdentityColumn {
    AlwaysGenerated,
    GeneratedByDefault,
}

impl<'de> Deserialize<'de> for IdentityColumn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "a" => Self::AlwaysGenerated,
            "d" => Self::GeneratedByDefault,
            i => panic!("Unexpected identity value `{:?}`", i)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneratedColumn {
    Stored,
}

impl<'de> Deserialize<'de> for GeneratedColumn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "s" => Self::Stored,
            g => panic!("Unexpected generated value `{:?}`", g)
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ColumnValue {
    pub comment: Option<String>,
    pub data_type: String,
    pub expression: Option<String>,
    pub generated: Option<GeneratedColumn>,
    pub identity: Option<IdentityColumn>,
    pub name: String,
    pub nullable: bool,
    pub position: Position,
}

impl ColumnValue {
    pub fn always_generated(&self) -> bool {
        self.identity == Some(IdentityColumn::AlwaysGenerated) ||
        self.generated == Some(GeneratedColumn::Stored)
    }

    pub fn required(&self) -> bool {
        !self.nullable && self.expression.is_none()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ForeignRef {
    pub oid: Oid,
    pub columns: Vec<Position>,
    pub match_type: ForeignKeyMatchType,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConstraintValue {
    pub name: String,
    pub columns: Vec<Position>,
    pub constraint_type: ConstraintType,
    pub expression: Option<String>,
    pub foreign_ref: Option<ForeignRef>,
}

impl ConstraintValue {
    pub fn column_constraint(&self) -> bool {
        self.columns.len() == 1
    }

    pub fn table_constraint(&self) -> bool {
        self.columns.len() > 1
    }
}

pub type Column = Json<ColumnValue>;
// pub type Constraint = Json<ConstraintValue>;
pub type ColumnConstraints = Json<HashMap<String, Vec<ConstraintValue>>>;


#[derive(Clone, Debug, Deserialize)]
pub struct Table {
    pub columns: Vec<Column>,
    pub comment: Option<String>,
    pub name: String,
    pub oid: Oid,
    pub schema: String,
}

impl Table {
    pub async fn load(pool: &PgPool, config: &Config, oid: u32) -> Option<Self> {
        sqlx::query_file_as!(
            Self,
            "queries/table-columns.sql",
            &config.scope.include,
            &config.scope.exclude,
            Oid(oid)
        )
            .fetch_optional(pool)
            .await
            .unwrap()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Constraints {
    pub table_oid: Oid,
    pub column_constraints: ColumnConstraints,
}

impl Constraints {
    pub async fn load(pool: &PgPool, config: &Config, oid: u32) -> Option<Self> {
        sqlx::query_file_as!(
            Self,
            "queries/table-constraints.sql",
            &config.scope.include,
            &config.scope.exclude,
            Oid(oid)
        )
            .fetch_optional(pool)
            .await
            .unwrap()
    }
}

// TODO: Not loving this, ripe for refactor
#[derive(Debug)]
pub struct TableWithConstraints {
    pub table: Table,
    pub constraints: Constraints,
}

impl TableWithConstraints {
    pub async fn load(pool: &PgPool, config: &Config, oid: u32) -> Option<Self> {
        let table = Table::load(pool, config, oid).await;
        let constraints = Constraints::load(pool, config, oid).await;

        match (table, constraints) {
            (Some(t), Some(c)) => Some(TableWithConstraints { table: t, constraints: c }),
            _ => None,
        }
    }
}
