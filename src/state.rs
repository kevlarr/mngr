use std::{collections::HashMap, env};

use serde::Deserialize;
use sqlx::{Executor, postgres::{PgPool, PgPoolOptions}, types::Json};

// TODO: Way too much happening in here
const CONFIG: &str = r#"
[database]

# Database tables to include, either unqualified or schema-qualified.
# Strings should be in a format compatible with `LIKE` comparisons.
#
# TODO: These follow same format as LIKE - should they be regex instead?
include = [
  "public.%",
]

# Database tables to exclude, primarily when more tables are included
# via `LIKE` than are ultimately desired.
exclude = [
  "public.jrny_revision",
]
"#;

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TableConfig {
    pub schema: Option<String>,
    pub name: String,
    pub lookup: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub tables: Vec<TableConfig>,
}




#[derive(Clone, Debug, Deserialize)]
pub struct ColumnRow {
    pub name: String,
    pub data_type: String,
    pub position: i32,
    pub nullable: bool,
    pub identity: Option<String>,
    pub generated: Option<String>,
    pub expression: Option<String>,
}

impl ColumnRow {
    pub fn always_generated(&self) -> bool {
        // TODO: Serialize as enums..?
        self.identity.as_deref() == Some("always") ||
        self.identity.as_deref() == Some("stored")
    }
}

pub type Column = Json<ColumnRow>;


#[derive(Clone, Debug, Deserialize)]
pub struct TableRow {
    pub name: String,
    pub columns: Vec<Json<ColumnRow>>,
}

pub type Table = Json<TableRow>;




// Unlike column and table "rows", the schema "row" itself is not
// exposed publicly, since it is more conveniently used via the
// `Schema` wrapper that adds a table lookup map.
#[derive(Clone, Debug, Deserialize)]
struct SchemaRow {
    name: String,
    tables: Vec<Table>,
}




#[derive(Clone, Debug)]
pub struct Schema {
    pub name: String,
    pub tables: Vec<Table>,
    map: HashMap<String, usize>,
}

impl Schema {
    fn get_table(&self, table_name: &str) -> Option<&Table> {
        self.tables.get(*self.map.get(table_name)?)
    }
}

impl From<SchemaRow> for Schema {
    fn from(schema: SchemaRow) -> Self {
        let map = schema.tables.iter()
            .enumerate()
            .map(|(i, table)| (table.name.clone() , i))
            .collect();

        Self { map, name: schema.name, tables: schema.tables }
    }
}

#[derive(Clone, Debug)]
pub struct Schemas {
    schemas: Vec<Schema>,
    map: HashMap<String, usize>,
}

impl Schemas {
    pub fn get_table(&self, schema_name: &str, table_name: &str) -> Option<&Table> {
        self.schemas.get(*self.map.get(schema_name)?)?
            .get_table(table_name)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Schema> {
        self.schemas.iter()
    }

    pub fn len(&self) -> usize {
        self.schemas.len()
    }
}

impl From<Vec<SchemaRow>> for Schemas {
    fn from(schemas: Vec<SchemaRow>) -> Self {
        let map = schemas.iter()
            .enumerate()
            .map(|(i, schema)| (schema.name.clone() , i))
            .collect();

        let schemas = schemas.into_iter()
            .map(|schema| Schema::from(schema))
            .collect();

        Self { schemas, map }
    }
}


#[derive(Clone, Debug)]
pub struct State {
    pub config: Config,
    pub pool: PgPool,
    pub schemas: Schemas,
}

impl State {
    pub async fn new() -> Self {
        let config: Config = toml::from_str(CONFIG).unwrap();

        // TODO: From an env file or argument
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .after_connect(|conn, _meta| Box::pin(async move {
                conn.execute("SET application_name = 'alpaca-admin'").await?;
                Ok(())
            }))
            .connect(&database_url)
            .await
            .unwrap();

        let schemas = sqlx::query_file_as!(
            SchemaRow,
            "queries/tables.sql",
            &config.database.include,
            &config.database.exclude
        )
            .fetch_all(&pool)
            .await
            .unwrap();


        State {
            config,
            pool,
            schemas: Schemas::from(schemas),
        }
    }
}
