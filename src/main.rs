use std::{collections::HashMap, env, io};

use actix_files::Files;
use actix_web::{
    middleware::DefaultHeaders,
    web::{Data, Form, Path, scope},
    App,
    Either,
    HttpResponse,
    HttpServer,
    get,
    post,
};
use maud::{html, DOCTYPE, Markup};
use serde::Deserialize;
use sqlx::Error as SqlError;

use admin::{state::*, ui};


const YEAR_IN_SECONDS: isize = 60 * 60 * 24 * 365;


#[derive(Deserialize)]
struct TablePath {
    schema: String,
    table: String,
}

fn page(state: &State, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        head {
            script src="/static/js/main.js" async {}
            link rel="stylesheet" type="text/css" href="/static/css/main.css";
        }
        body {
            c-sidebar {
                h1 { (state.schemas.database_name()) }
                nav {
                    @for schema in state.schemas.iter() {
                        section {
                            h2 { (schema.name) }
                            menu {
                                @for table in &schema.tables {
                                    li {
                                        a
                                            href=(format!("/tables/{}/{}/records", schema.name, table.name))
                                            data-schema=(schema.name)
                                            data-table=(table.name)
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


#[get("/debug/state")]
async fn get_state(state: Data<State>) -> Markup {
    let state = format!("{:#?}", state);

    html! {
        pre { (state) }
    }
}


#[get("/tables")]
async fn get_tables(state: Data<State>) -> Markup {
    page(&state, html! {
    })
}


#[get("/tables/{schema}/{table}/records")]
async fn get_table_records(path: Path<TablePath>, state: Data<State>) -> Markup {
    // TODO: Implement an extractor for this
    match state.schemas.get_table(&path.schema, &path.table) {
        Some(table) => render_records(&state, &path.schema, table).await,
        None => not_found(&state),
    }
}


#[get("/tables/{schema}/{table}/records/new")]
async fn get_table_records_new(path: Path<TablePath>, state: Data<State>) -> Markup {
    match state.schemas.get_table(&path.schema, &path.table) {
        Some(table) => render_new_record(&state, &path.schema, table, None),
        None => not_found(&state),
    }
}

#[post("/tables/{schema}/{table}/records/new")]
async fn post_table_records_new(
    path: Path<TablePath>,
    state: Data<State>,
    form: Form<HashMap<String, String>>,
) -> Either<HttpResponse, Markup> {
    match state.schemas.get_table(&path.schema, &path.table) {
        Some(table) => create_new_record(&state, &path.schema, table, &form).await,
        None => Either::Right(not_found(&state)),
    }
}


async fn render_records(state: &State, schema_name: &str, table: &Table) -> Markup {
    // Assumes query returns columns sorted by their ordinal position, in which case
    // also assume the first column is probably the primary key and should be used
    // for ordering by default.
    let first_col = table.columns.first().unwrap();

    // TODO: This has become a common pattern, so.. abstract. Maybe think about
    // adding this as a pre-generated field on the `Table` struct?
    let columns = table.columns.iter()
        .map(|c| format!("{}::text", c.name))
        .collect::<Vec<_>>()
        .join(", ");

    let statement = format!(r#"
        SELECT {} FROM "{}"."{}"
        ORDER BY "{}"::{} ASC
        LIMIT 50
        "#,
        columns,
        schema_name,
        table.name,
        first_col.name,
        first_col.meta.data_type,
    );

    let rows = sqlx::query(&statement)
        .fetch_all(&state.pool)
        .await
        .unwrap();

    let ui_table = ui::table::Table::new(
        table.columns.as_slice(),
        rows,
    );

    page(state, html! {
        header {
            h2 { (table.name) " :: Records" }
            menu {
                li {
                    a href=(format!("/tables/{}/{}/records/new", schema_name, table.name)) {
                        "New Record"
                    }
                }
            }
        }
        (ui_table)
    })
}


fn render_new_record(state: &State, schema_name: &str, table: &Table, error: Option<SqlError>) -> Markup {
    let mut ui_form = ui::form::Form::from(table.columns.as_slice())
        .method("post")
        .action(&format!("/tables/{}/{}/records/new", schema_name, table.name));

    if let Some(e) = error {
        ui_form = ui_form.error(e);
    }

    page(state, html! {
        header {
            h2 { (table.name) " :: New Record" }
            menu {
                li {
                    a href=(format!("/tables/{}/{}/records", schema_name, table.name)) {
                        "Back to Records"
                    }
                }
            }
        }
        (ui_form)
    })
}
async fn create_new_record(
    state: &State,
    schema_name: &str,
    table: &Table,
    form_data: &HashMap<String, String>,
) -> Either<HttpResponse, Markup> {
    let keys = form_data.keys()
        .map(|k| &**k)
        .collect::<Vec<_>>();

    let columns = keys.join(", ");

    // TODO: Optimize this with a map lookup of key -> table column
    let mut bind_variables = Vec::new();

    for (i, key) in keys.iter().enumerate() {
        let column = table.columns.iter().find(|c| c.name == *key).unwrap();
        let variable = format!("${}::{}", i + 1, column.meta.data_type);

        bind_variables.push(variable);
    }

    let statement = format!(r#"
        INSERT INTO "{}"."{}" ({})
            VALUES ({})
        "#,
        schema_name,
        table.name,
        columns,
        bind_variables.join(", "),
    );

    let mut query = sqlx::query(&statement);

    for key in keys {
        query = query.bind(form_data.get(key).unwrap());
    }

    match query.execute(&state.pool).await {
        Ok(_) => Either::Left(HttpResponse::SeeOther()
            .insert_header(("Location", format!("/tables/{}/{}/records", schema_name, table.name).as_str()))
            .finish()
        ),
        Err(e) => Either::Right(render_new_record(state, schema_name, table, Some(e)))
    }
}


fn not_found(state: &State) -> Markup {
    page(state, html! {
        h1 { "Not found" }
    })
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
            .service(get_tables)
            .service(get_table_records)
            .service(get_table_records_new)
            .service(post_table_records_new)
    };

    HttpServer::new(app_builder)
        .bind(("127.0.0.1", 3001))?
        .run()
        .await
}
