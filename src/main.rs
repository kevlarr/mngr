use actix_files::Files;
use actix_web::{
    middleware::DefaultHeaders,
    web::{Data, Form, Path, Query, scope},
    App,
    Either,
    HttpResponse,
    HttpServer,
    get,
    post,
};
use maud::{html, DOCTYPE, Markup};
use mngr::{state::*, db, ui};
use serde::Deserialize;
use sqlx::Error as SqlError;
use std::{collections::HashMap, env, io};

const YEAR_IN_SECONDS: isize = 60 * 60 * 24 * 365;

#[derive(Deserialize)]
struct RecordsPath {
    table_oid: u32,
}

#[derive(Deserialize)]
struct RecordsParams {
    page: Option<i64>,
    sort_column: Option<String>,
    sort_direction: Option<String>,
}

#[derive(Deserialize)]
struct RecordPath {
    table_oid: u32,
    record_id: i64,
}

async fn page(state: &State, content: Markup) -> Markup {
    let schemas = mngr::db::Schemas::load(&state.pool, &state.config).await;

    html! {
        (DOCTYPE)
        head {
            script src="/static/js/main.js" async {}
            link rel="stylesheet" type="text/css" href="/static/css/main.css";
        }
        body {
            c-sidebar {
                h1 { "mngr" }
                nav {
                    @let show_schema = schemas.len() > 1;

                    @for schema in schemas.iter() {
                        section {
                            @if show_schema {
                                h2 { (schema.name) }
                            }
                            menu {
                                @for table in &schema.tables {
                                    li {
                                        a
                                            href=(format!("/tables/{}/records", table.oid.0))
                                            data-table-oid=(table.oid.0)
                                        {
                                            (table.name)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            c-content {
                (content)
            }
        }
    }
}

async fn records_page(
    state: &State,
    table: &db::Table,
    content: Markup,
) -> Markup {
    page(state, html! {
        header {
            h2 { (table.name) }
            menu class="tabs" {
                li {
                    a href=(format!("/tables/{}/records", table.oid.0)) {
                        "All Records"
                    }
                }
                li {
                    a href=(format!("/tables/{}/records/new", table.oid.0)) {
                        "New Record"
                    }
                }
            }
        }
        (content)
    }).await
}

#[get("/debug/state")]
async fn get_state(state: Data<State>) -> Markup {
    let state = format!("{:#?}", state);

    html! {
        pre { (state) }
    }
}

#[get("/tables/{table_oid}/records")]
async fn get_table_records(
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
async fn get_table_records_new(
    path: Path<RecordsPath>,
    state: Data<State>,
) -> Markup {
    match load_table(&state, path.table_oid).await {
        Some(table) => render_new_record(&state, &table, None).await,
        None => not_found(&state).await,
    }
}

#[post("/tables/{table_oid}/records/new")]
async fn post_table_records_new(
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
async fn get_table_record_edit(
    path: Path<RecordPath>,
    state: Data<State>,
) -> Markup {
    match load_table(&state, path.table_oid).await {
        Some(table) => render_edit_record(&state, &table, path.record_id, None).await,
        None => not_found(&state).await,
    }
}

#[post("/tables/{table_oid}/records/{record_id}/edit")]
async fn post_table_record_edit(
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

async fn render_records(
    state: &State,
    table: &db::Table,
    params: &RecordsParams,
) -> Markup {
    let sort_column = match &params.sort_column {
        Some(c1) => {
            table.columns.iter().find(|c2| c1 == &c2.name).unwrap()
        },
        None => {
            // Assumes query returns columns sorted by their ordinal position, in which case
            // also assume the first column is probably the primary key and should be used
            // for ordering by default.
            table.columns.first().unwrap()
        },
    };

    let sort_direction = params.sort_direction.as_ref().map_or("asc", |sd| sd.as_str());

    // TODO: This has become a common pattern, so.. abstract. Maybe think about
    // adding this as a pre-generated field on the `Table` struct?
    let columns = table.columns.iter()
        .map(|c| format!("\"{}\"::text", c.name))
        .collect::<Vec<_>>()
        .join(", ");

    // TODO: Incorporate limit & pagination params
    let statement = format!(r#"
        SELECT {} FROM "{}"."{}"
        ORDER BY "{}"::{} {}
        LIMIT 50
        "#,
        columns,
        table.schema,
        table.name,
        sort_column.name,
        sort_column.data_type,
        sort_direction,
    );

    let result = sqlx::query(&statement)
        .fetch_all(&state.pool)
        .await;

    match result {
        Ok(rows) => {
            let ui_table = ui::table::Table::new(&table, rows);
            records_page(state, table, html! {
                (ui_table)
            }).await
        }
        Err(e) => {
            records_page(state, table, html! {
                pre {
                    (statement)
                }
                pre {
                    (format!("{:#?}", e))
                }
            }).await
        }
    }

}


async fn render_new_record(
    state: &State,
    table: &db::Table,
    error: Option<SqlError>,
) -> Markup {
    let mut ui_form = ui::form::Form::from(table.columns.as_slice())
        .method("post")
        .action(&format!("/tables/{}/records/new", table.oid.0));

    if let Some(e) = error {
        ui_form = ui_form.error(e);
    }

    records_page(state, table, html! { (ui_form ) }).await
}


async fn render_edit_record(
    state: &State,
    table: &db::Table,
    record_id: i64, // TODO: Dynamic primary key column, not just "id"
    error: Option<SqlError>,
) -> Markup {
    // TODO: Abstract, this is duplicate
    let columns = table.columns.iter()
        .map(|c| format!("\"{}\"::text", c.name))
        .collect::<Vec<_>>()
        .join(", ");

    // TODO: Don't rely on `id` field
    let statement = format!(r#"
        SELECT {} FROM "{}"."{}"
        WHERE id = $1
        "#,
        columns,
        table.schema,
        table.name,
    );

    let result = sqlx::query(&statement)
        .bind(record_id)
        .fetch_one(&state.pool)
        .await;

    match result {
        Ok(row) => {
            let mut ui_form = ui::form::Form::from(table.columns.as_slice())
                .method("post")
                .action(&format!("/tables/{}/records/{}/edit", table.oid.0, record_id))
                .row(&row);

            if let Some(e) = error {
                ui_form = ui_form.error(e);
            }

            records_page(state, table, html! {
                (ui_form)
            }).await
        }
        Err(e) => {
            records_page(state, table, html! {
                pre {
                    (statement)
                }
                pre {
                    (format!("{:#?}", e))
                }
            }).await
        }
    }
}


async fn create_new_record(
    state: &State,
    table: &db::Table,
    form_data: &HashMap<String, String>,
) -> Either<HttpResponse, Markup> {
    let mut columns = Vec::new();
    let mut bind_variables = Vec::new();
    let mut bind_params = Vec::new();
    let mut position = 1;

    for (key, value) in form_data.iter() {
        if value.len() == 0 { continue; }

        // TODO: Optimize this with a map lookup of key -> table column
        let column = table.columns.iter().find(|c| c.name == *key).unwrap();

        columns.push(key.to_owned());
        bind_variables.push(format!("${}::{}", position, column.data_type));
        bind_params.push(value.to_owned());

        position += 1;
    }

    let statement = format!(r#"
        INSERT INTO "{}"."{}" ({})
            VALUES ({})
        "#,
        table.schema,
        table.name,
        columns.join(", "),
        bind_variables.join(", "),
    );

    let mut query = sqlx::query(&statement);

    for param in bind_params {
        query = query.bind(param);
    }

    match query.execute(&state.pool).await {
        Ok(_) => Either::Left(HttpResponse::SeeOther()
            .insert_header(("Location", format!("/tables/{}/records", table.oid.0).as_str()))
            .finish()
        ),
        Err(e) => Either::Right(render_new_record(state, table, Some(e)).await)
    }
}


async fn update_record(
    state: &State,
    table: &db::Table,
    record_id: i64,
    form_data: &HashMap<String, String>,
) -> Either<HttpResponse, Markup> {
    let keys = form_data.keys()
        .map(|k| &**k)
        .collect::<Vec<_>>();

    let mut props = Vec::new();

    for (i, key) in keys.iter().enumerate() {
        let column = table.columns.iter().find(|c| c.name == *key).unwrap();

        props.push(format!("{} = ${}::{}", column.name, i + 1, column.data_type));
    }

    // TODO: Need to know primary key column, not just assume id
    let statement = format!(r#"
        UPDATE "{}"."{}" SET {} WHERE id = ${}
        "#,
        table.schema,
        table.name,
        props.join(", "),
        keys.len() + 1,
    );

    let mut query = sqlx::query(&statement);

    for key in keys {
        query = query.bind(form_data.get(key).unwrap());
    }

    query = query.bind(&record_id);

    match query.execute(&state.pool).await {
        Ok(_) => Either::Left(HttpResponse::SeeOther()
            .insert_header(("Location", format!("/tables/{}/records/{}/edit", table.oid.0, record_id).as_str()))
            .finish()),

        // FIXME: This shouldn't use same 'edit' function because it reloads the record
        // and loses all form data
        Err(e) => Either::Right(render_edit_record(state, table, record_id, Some(e)).await)
    }
}


async fn not_found(state: &State) -> Markup {
    page(state, html! {
        h1 { "Not found" }
    }).await
}


#[actix_web::main]
async fn main() -> io::Result<()> {
    let state = State::new().await;

    let app_builder = move || {
        let static_scope = scope("/static")
            .service(
                Files::new("/", env::var("STATIC_PATH").expect("STATIC_PATH not set"))
                    .show_files_listing() // Enable subdirectories
                    .use_last_modified(true)
            )
            .wrap(
                DefaultHeaders::new()
                    .add(("Cache-Control", format!("max-age={YEAR_IN_SECONDS}").as_str()))
            );

        App::new()
            .app_data(Data::new(state.clone()))
            .service(static_scope)
            .service(get_state)
            .service(get_table_records)
            .service(get_table_records_new)
            .service(post_table_records_new)
            .service(get_table_record_edit)
            .service(post_table_record_edit)
    };

    HttpServer::new(app_builder)
        .bind(("127.0.0.1", 3001))?
        .run()
        .await
}
