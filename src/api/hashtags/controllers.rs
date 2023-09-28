use actix_web::{
  web::{self, Query},
  HttpResponse,
};
use deadpool_postgres::{Client, Pool};
use serde_json::json;

use crate::api::handler_utils::NoDBClient;

use super::models::FetchTrendingHashtags;

pub async fn get_trending_hashtags(
  db_pool: web::Data<Pool>,
  body: Query<FetchTrendingHashtags<NoDBClient>>,
) -> HttpResponse {
  let db_client_res = db_pool.get().await;

  if let Err(e) = db_client_res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e.to_string(),
    }));
  }

  let db_client: Client = db_client_res.unwrap();

  let res = body.into_inner().add_db_client(&db_client).fetch().await;

  match res {
    Ok(data) => HttpResponse::Ok().json(json!({
      "success": true,
      "data": data
    })),

    Err((s, v)) => HttpResponse::Ok().status(s).json(json!({
      "success": false,
      "message": v["message"],
      "error": v
    })),
  }
}
