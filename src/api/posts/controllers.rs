use actix_web::{web, HttpResponse};
use deadpool_postgres::{Client, Pool};
use serde_json::json;

use super::models;
use crate::api::UserAuth;

pub async fn create_post(
  user_detail: UserAuth,
  db_pool: web::Data<Pool>,
  body: web::Json<models::CreatePostDetails>,
) -> HttpResponse {
  let db_client_res = db_pool.get().await;

  if let Err(e) = db_client_res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e.to_string(),
    }));
  }

  let db_client = db_client_res.unwrap();

  let res = body.into_inner().add_db_client(&db_client).create_post();

  if let Err(e) = res {
    return HttpResponse::Ok().status(e.0).json(json!({
      "success": false,
      "message": e.1["message"],
      "error": e.1
    }));
  }

  HttpResponse::Ok().finish()
}
