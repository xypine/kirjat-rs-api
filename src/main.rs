use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{get, middleware, web::Data, App, HttpServer, Responder};

use kirjat::Cache;

mod handlers;

pub struct AppState {
    pub cache: Mutex<Cache>,
}

#[get("/")]
async fn greet(app_state: Data<AppState>) -> impl Responder {
    let cache = &app_state.cache;
    format!("{} items cached", cache.lock().unwrap().entry_count())
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let cache = Cache::new(10_000);
    let app_state = Data::new(AppState {
        cache: Mutex::new(cache),
    });

    let http_bind = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    println!("Starting to listen on {}:...", http_bind);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Enable CORS
            .wrap(Cors::permissive())
            // Enable logger
            .wrap(middleware::Logger::default())
            .service(greet)
            .service(handlers::api_v3::query_v3)
    })
    .bind(http_bind)?
    .run()
    .await
}
