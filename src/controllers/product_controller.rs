use crate::models::product::Product;
use accept_language::intersection_with_quality;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use serde::Deserialize;
use serde_json::json;
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Deserialize)]
pub struct QueryParams {
    search: Option<String>,
}

pub async fn translated_products_handler(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    query_params: web::Query<QueryParams>,
) -> impl Responder {
    // Extract "Accept-Language" header
    let header_value = req
        .headers()
        .get("Accept-Language")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Define the supported languages
    let supported_languages = ["fr", "en", "zh"]; // French, English, Chinese

    // Find the intersection with quality factor to determine the best match
    let common_languages = intersection_with_quality(header_value, &supported_languages);

    // Select the highest quality language from the intersection result
    let selected_language = common_languages.first().map_or("en", |(lang, _)| lang);

    // Extract search query if available
    let search_query = query_params.search.as_deref();

    let connection_result = pool.get();

    let mut connection = match connection_result {
        Ok(conn) => conn,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Error getting DB connection from pool"}))
        }
    };
    match Product::get_products_grouped_by_category(&mut *connection, selected_language, search_query) {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
