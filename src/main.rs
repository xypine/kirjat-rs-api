use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{get, http::Error, middleware, web::Data, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

use kirjat::Cache;

mod handlers;

pub struct AppState {
    pub cache: Mutex<Cache>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Stats {
    pub pages_cached: usize,
}
#[derive(Serialize, Debug)]
pub struct Greet {
    pub endpoints: [&'static str; 4],
    pub stats: Stats,
    pub app_version: &'static str,
    pub app_license: &'static str,
}
#[get("/")]
async fn greet(app_state: Data<AppState>) -> Result<HttpResponse, Error> {
    let cache = &app_state.cache;
    let pages_cached = cache.lock().unwrap().entry_count() as usize;
    let out = Greet {
        stats: Stats { pages_cached },
        endpoints: [
            "/",
            "/api/v3/search",
            "/api/v3/search/source/{source}",
            "/api/v3/cached_pages_v3",
        ],
        app_version: env!("CARGO_PKG_VERSION"),
        app_license: env!("CARGO_PKG_LICENSE"),
    };
    return Ok::<HttpResponse, Error>(HttpResponse::Ok().json(out));
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
            .service(handlers::api_v3::query_v3_source)
            .service(handlers::api_v3::cached_pages_v3)
    })
    .bind(http_bind)?
    .run()
    .await
}
