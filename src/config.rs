use serde::Deserialize;
use std::fs;

#[derive(Clone, Debug, Deserialize)]
pub struct ScopeConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TableConfig {
    // TODO: If this is optional, there needs to be an error at application
    // startup describing ambiguous table if there are multiple loaded from
    // the initial state SQL query with the same name in different schemas
    pub schema: Option<String>,
    pub table: String,
    pub description: Option<String>,
    pub lookup: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub scope: ScopeConfig,
    pub tables: Option<Vec<TableConfig>>,
}

impl Config {
    pub fn load(filepath: &str) -> Config {
        let contents = fs::read_to_string(filepath).unwrap();
        toml::from_str(&contents).unwrap()
    }
}
