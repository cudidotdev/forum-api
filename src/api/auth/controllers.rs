use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;

use super::models;

pub async fn create_account(
  body: Json<models::CreateAccountReq>,
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

  let body = body.into_inner().validate();

  if let Err(err) = body {
    return HttpResponse::BadRequest().json(json!({
      "success": false,
      "message": err["message"],
      "error": err
    }));
  }

  let body = body.unwrap();

  let res = body.insert_to_db(&db_client).await;

  if let Err(err) = res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": err,
    }));
  }

  HttpResponse::Ok().json(json!({ "success": true, "data": res.unwrap() }))
}
