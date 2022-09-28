use std::sync::Mutex;

use actix_web::{http::Error, post, web::Data, HttpResponse};
use kirjat::Cache;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryV2 {
    pub queryall: Option<String>,
    pub queryjamera: Option<String>,
    pub querysanomapro: Option<String>,
}
#[derive(Deserialize, Serialize, Debug)]
pub enum QueryV2Result {
    Ok(Vec<kirjat::structs::kirja::Kirja>),
    Error(String),
}
#[derive(Deserialize, Serialize, Debug)]
pub struct QueryV2Return {
    pub code: usize,
    pub result: Vec<QueryV2Result>,
}
#[post("/api/v2")]
pub async fn query_v2(
    query: actix_web::web::Form<QueryV2>,
    cache: Data<Mutex<Cache>>,
) -> Result<HttpResponse, Error> {
    let mut cache_live = cache.lock().unwrap();
    let mut cache_mutable = cache_live.clone();

    println!("{} items in cache", cache_mutable.entry_count());

    if let Some(queryall) = &query.queryall {
        let mut result: Vec<QueryV2Result> = vec![];

        let book_names: Vec<&str> = queryall.split("\n").collect();
        for book_name in book_names {
            let name = book_name.to_string();
            let queryresult =
                kirjat::search_book_from_all_sources(&name, &Some(&mut cache_mutable)).await;
            if let Ok(books) = queryresult {
                result.push(QueryV2Result::Ok(books));
            } else if let Err(error) = queryresult {
                result.push(QueryV2Result::Error(error.to_string()));
            }
        }

        // Update cache
        *cache_live = cache_mutable;

        let out = QueryV2Return { code: 200, result };

        return Ok(HttpResponse::Ok().json(out));
    }

    Ok(HttpResponse::Ok().body("Nothing interesting happened. Try again."))
}
