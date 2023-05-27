use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;

use super::models;

pub async fn create_account(
  body: Json<models::CreateAccountDetails>,
  db_pool: Data<Pool>,
) -> HttpResponse {
  let db_client_res = db_pool.get().await;

  if let Err(e) = db_client_res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e.to_string(),
    }));
  }

  let db_client = db_client_res.unwrap();

  let res = body
    .into_inner()
    .add_db_client(&db_client)
    .insert_to_db()
    .await;

  if let Err(err) = res {
    return HttpResponse::BadRequest().json(json!({
      "success": false,
      "message": err["message"],
      "error": err
    }));
  }

  HttpResponse::Ok().json(json!({ "success": true, "id": res.unwrap() }))
}

pub async fn login(body: Json<models::LoginDetails>, db_pool: Data<Pool>) -> HttpResponse {
  let db_client_res = db_pool.get().await;

  if let Err(e) = db_client_res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e.to_string(),
    }));
  }

  let db_client = db_client_res.unwrap();

  let body = body.into_inner().add_db_client(&db_client).validate().await;

  if let Err(err) = body {
    return HttpResponse::BadRequest().json(json!({
      "success": false,
      "message": err["message"],
      "error": err
    }));
  }

  let body = body.unwrap();

  HttpResponse::Ok().json(json!({
    "success": true,
    "data": {
      "access_token": body.to_jwt()
    }
  }))
}
