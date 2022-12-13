use crate::config::Config;
use sqlx::{Executor, postgres::{PgPool, PgPoolOptions}};
use std::env;

#[derive(Clone, Debug)]
pub struct State {
    pub config: Config,
    pub pool: PgPool,
}

impl State {
    pub async fn new() -> Self {
        let config = Config::load("test/mngr.toml");

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

        State {
            config,
            pool,
        }
    }
}
