use actix_files::Files;
use actix_web::{
    middleware::DefaultHeaders,
    web::{Data, scope},
    App,
    HttpServer,
};
use mngr::{state::State, routes::*};
use std::{env, io};

const YEAR_IN_SECONDS: isize = 60 * 60 * 24 * 365;

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
            .service(get_table_debug)
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
