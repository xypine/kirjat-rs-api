use std::collections::HashMap;

use actix_web::{
    get,
    http::Error,
    web::{Data, Path, Query},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize)]
pub struct QueryV3 {
    names: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub enum QueryV3Result {
    #[serde(rename(serialize = "results"))]
    Ok(Vec<kirjat::structs::kirja::Kirja>),
    #[serde(rename(serialize = "error"))]
    Error(String),
}
#[get("/api/v3/search")]
pub async fn query_v3(
    app_state: Data<AppState>,
    query: Query<QueryV3>,
) -> Result<HttpResponse, Error> {
    let cache = &app_state.cache;
    let mut cache_live = cache.lock().unwrap().clone();

    let mut out = HashMap::new();

    let book_names: Vec<&str> = query.names.split(",").collect();
    for book_name in book_names {
        let name = book_name.to_string();
        let queryresult = kirjat::search_book_from_all_sources(&name, &Some(&mut cache_live)).await;
        match queryresult {
            Ok(books) => {
                out.insert(book_name, QueryV3Result::Ok(books));
            }
            Err(error) => match error {
                _ => {
                    out.insert(book_name, QueryV3Result::Error(error.to_string()));
                }
            },
        }
    }

    // Update cache
    *cache.lock().unwrap() = cache_live;

    return Ok(HttpResponse::Ok().json(out));
}

#[get("/api/v3/search/source/{source}")]
pub async fn query_v3_source(
    app_state: Data<AppState>,
    query: Query<QueryV3>,
    source: Path<kirjat::sources::Sources>,
) -> Result<HttpResponse, Error> {
    let cache = &app_state.cache;
    let mut cache_live = cache.lock().unwrap().clone();

    let mut out = HashMap::new();

    let book_names: Vec<&str> = query.names.split(",").collect();
    for book_name in book_names {
        let name = book_name.to_string();
        let queryresult = kirjat::search_book(&name, *source, &Some(&mut cache_live)).await;
        match queryresult {
            Ok(books) => {
                out.insert(book_name, QueryV3Result::Ok(books));
            }
            Err(error) => match error {
                _ => {
                    out.insert(book_name, QueryV3Result::Error(error.to_string()));
                }
            },
        }
    }

    // Update cache
    *cache.lock().unwrap() = cache_live;

    return Ok(HttpResponse::Ok().json(out));
}
