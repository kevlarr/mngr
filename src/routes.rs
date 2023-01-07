use actix_web::{
    web::{Data, Form, Path, Query},
    Either,
    HttpResponse,
    get,
    post,
};
use crate::{db, renderers::*, state::State};
use maud::{html, Markup};
use serde::Deserialize;
use std::{collections::HashMap};

#[derive(Deserialize)]
pub struct RecordsPath {
    pub table_oid: u32,
}

#[derive(Deserialize)]
pub struct RecordsParams {
    pub page: Option<i64>,
    pub sort_column: Option<String>,
    pub sort_direction: Option<String>,
}

#[derive(Deserialize)]
pub struct RecordPath {
    pub table_oid: u32,
    pub record_id: i64,
}

#[get("/debug/state")]
pub async fn get_state(state: Data<State>) -> Markup {
    let state = format!("{:#?}", state);

    html! {
        pre { (state) }
    }
}

#[get("/tables/{table_oid}/debug")]
pub async fn get_table_debug(
    path: Path<RecordsPath>,
    state: Data<State>,
 ) -> Markup {
    match load_table(&state, path.table_oid).await {
        Some(table) => records_page(&state, &table, html! {
            pre {
                (format!("{:#?}", table))
            }
        }).await,
        None => not_found(&state).await,
    }
}

#[get("/tables/{table_oid}/records")]
pub async fn get_table_records(
    path: Path<RecordsPath>,
    params: Query<RecordsParams>,
    state: Data<State>,
) -> Markup {
    // TODO: Implement an extractor for this
    match load_table(&state, path.table_oid).await {
        Some(table) => render_records(&state, &table, &params).await,
        None => not_found(&state).await,
    }
}

#[get("/tables/{table_oid}/records/new")]
pub async fn get_table_records_new(
    path: Path<RecordsPath>,
    state: Data<State>,
) -> Markup {
    match load_table(&state, path.table_oid).await {
        Some(table) => render_new_record(&state, &table, None).await,
        None => not_found(&state).await,
    }
}

#[post("/tables/{table_oid}/records/new")]
pub async fn post_table_records_new(
    path: Path<RecordsPath>,
    state: Data<State>,
    form: Form<HashMap<String, String>>,
) -> Either<HttpResponse, Markup> {
    match load_table(&state, path.table_oid).await {
        Some(table) => create_new_record(&state, &table, &form).await,
        None => Either::Right(not_found(&state).await),
    }
}

#[get("/tables/{table_oid}/records/{record_id}/edit")]
pub async fn get_table_record_edit(
    path: Path<RecordPath>,
    state: Data<State>,
) -> Markup {
    match load_table(&state, path.table_oid).await {
        Some(table) => render_edit_record(&state, &table, path.record_id, None).await,
        None => not_found(&state).await,
    }
}

#[post("/tables/{table_oid}/records/{record_id}/edit")]
pub async fn post_table_record_edit(
    path: Path<RecordPath>,
    state: Data<State>,
    form: Form<HashMap<String, String>>,
) -> Either<HttpResponse, Markup> {
    match load_table(&state, path.table_oid).await {
        Some(table) => update_record(&state, &table, path.record_id, &form).await,
        None => Either::Right(not_found(&state).await),
    }
}

async fn load_table(
    state: &State,
    table_oid: u32,
) -> Option<db::Table> {
    db::Table::load(&state.pool, &state.config, table_oid).await
}
