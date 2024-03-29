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
pub struct QueryV3Result {
    pub results: Vec<kirjat::structs::kirja::Kirja>,
    pub errors: Vec<String>,
}
#[get("/api/v3/search")]
pub async fn query_v3(
    app_state: Data<AppState>,
    query: Query<QueryV3>,
) -> Result<HttpResponse, Error> {
    let cache = &app_state.cache;
    let mut cache_live = cache.lock().unwrap().clone();

    let mut out = HashMap::new();

    let book_names: Vec<&str> = query.names.split(",").map(|name| name.trim()).collect();
    for book_name in book_names {
        let name = book_name.to_string();
        let queryresult = kirjat::search_book_from_all_sources(&name, &Some(&mut cache_live)).await;
        let mut results = vec![];
        let mut errors = vec![];
        for result in queryresult {
            match result {
                Ok(mut books) => {
                    results.append(&mut books);
                    sort(&mut results, name.clone());
                }
                Err(error) => match error {
                    _ => {
                        errors.push(error.to_string());
                    }
                },
            }
        }
        out.insert(book_name, QueryV3Result { results, errors });
    }

    // Update cache
    *cache.lock().unwrap() = cache_live;

    return Ok(HttpResponse::Ok().json(out));
}

#[get("/api/v3/search/source/{source}")]
pub async fn query_v3_source(
    app_state: Data<AppState>,
    query: Query<QueryV3>,
    source: Path<kirjat::sources::BuiltInSource>,
) -> Result<HttpResponse, Error> {
    let cache = &app_state.cache;
    let mut cache_live = cache.lock().unwrap().clone();

    let mut out = HashMap::new();

    let book_names: Vec<&str> = query.names.split(",").map(|name| name.trim()).collect();
    for book_name in book_names {
        let name = book_name.to_string();
        let queryresult = kirjat::search_book(
            &name,
            &kirjat::sources::get_instance(*source),
            &Some(&mut cache_live),
        )
        .await;
        let mut results = vec![];
        let mut errors = vec![];
        match queryresult {
            Ok(mut books) => {
                results.append(&mut books);
                sort(&mut results, name.clone());
            }
            Err(error) => match error {
                _ => {
                    errors.push(error.to_string());
                }
            },
        }
        out.insert(book_name, QueryV3Result { results, errors });
    }

    // Update cache
    *cache.lock().unwrap() = cache_live;

    return Ok(HttpResponse::Ok().json(out));
}

#[get("/api/v3/cached_pages")]
pub async fn cached_pages_v3(app_state: Data<AppState>) -> Result<HttpResponse, Error> {
    let cache = &app_state.cache;
    let cache_live = cache.lock().unwrap().clone();

    return Ok(HttpResponse::Ok().json(
        cache_live
            .iter()
            .map(|(k, _v)| (*k).clone())
            .collect::<Vec<String>>(),
    ));
}

pub fn score_result(result: &kirjat::structs::kirja::Kirja, query: &String) -> f64 {
    ((1.0 - (strsim::normalized_damerau_levenshtein(&result.name, query).abs()) * 9.0)
        + (result
            .get_min_price()
            .unwrap_or(kirjat::structs::currency::Currency { euro_cents: 20_000 })
            .to_euros()
            * 0.1))
        / 10.0
}

pub fn sort(books: &mut Vec<kirjat::structs::kirja::Kirja>, perfect: String) {
    books.sort_by(|a, b| score_result(a, &perfect).total_cmp(&score_result(b, &perfect)));
}
