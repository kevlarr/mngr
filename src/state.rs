use std::{collections::HashMap, env};

use serde::Deserialize;
use sqlx::{Executor, postgres::{PgPool, PgPoolOptions}, types::Json};


#[derive(Clone, Debug, Deserialize)]
pub struct ColumnMeta {
    pub column_default: Option<String>,
    pub data_type: String,
    pub generation_expression: Option<String>,
    pub is_nullable: String,
    pub is_generated: String,
    pub ordinal_position: i32,

    pub is_identity: String,
    pub identity_generation: Option<String>,
    // pub identity_start: Option<String>,
    // pub identity_increment: Option<String>,
    // pub identity_maximum: Option<String>,
    // pub identity_minimum: Option<String>,
    // pub identity_cycle: Option<String>,

    // character_maximum_length: String,
    // character_octet_length: String,

    // table_catalog: String,
    // table_schema: String,
    // table_name: String,
    // numeric_precision: String,
    // numeric_precision_radix: String,
    // numeric_scale: String,
    // datetime_precision: String,
    // interval_type: String,
    // interval_precision: String,
    // character_set_catalog: String,
    // character_set_schema: String,
    // character_set_name: String,
    // collation_catalog: String,
    // collation_schema: String,
    // collation_name: String,
    // domain_catalog: String,
    // domain_schema: String,
    // domain_name: String,
    // udt_catalog: String,
    // udt_schema: String,
    // udt_name: String,
    // scope_catalog: String,
    // scope_schema: String,
    // scope_name: String,
    // maximum_cardinality: String,
    // dtd_identifier: String,
    // is_self_referencing: String,
    // is_updatable: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ColumnRow {
    pub name: String,
    pub meta: Json<ColumnMeta>,
}

impl ColumnRow {
    pub fn always_generated(&self) -> bool {
        self.meta.is_generated == "ALWAYS" ||
        self.meta.identity_generation.as_deref() == Some("ALWAYS")

    }
}

pub type Column = Json<ColumnRow>;




#[derive(Clone, Debug, Deserialize)]
pub struct TableMeta {
    pub table_catalog: String, // Database name
    pub table_type: String,

    // table_schema: String,
    // table_name: String,
    // self_referencing_column_name: String,
    // reference_generation: String,
    // user_defined_type_catalog: String,
    // user_defined_type_schema: String,
    // user_defined_type_name: String,
    // is_insertable_into: String,
    // is_typed: String,
    // commit_action: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TableRow {
    pub name: String,
    pub meta: Json<TableMeta>,
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
    pub fn database_name(&self) -> &str {
        &self.schemas[0].tables[0].meta.table_catalog
    }

    pub fn get_table(&self, schema_name: &str, table_name: &str) -> Option<&Table> {
        self.schemas.get(*self.map.get(schema_name)?)?
            .get_table(table_name)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Schema> {
        self.schemas.iter()
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
    pub pool: PgPool,
    pub schemas: Schemas,
}

impl State {
    pub async fn new() -> Self {
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

        // TODO: Parameterize this, maybe even an external TOML config file
        let tablenames = vec![
            "public.cities".to_owned(),
            "public.city_zips".to_owned(),
            "public.exids".to_owned(),
            "public.states".to_owned(),
            "public.zips".to_owned(),
        ];

        let schemas = sqlx::query_file_as!(SchemaRow, "queries/tables.sql", tablenames.as_slice())
            .fetch_all(&pool)
            .await
            .unwrap();

        State { pool, schemas: Schemas::from(schemas) }
    }
}
