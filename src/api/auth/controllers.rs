use actix_web::{web::Json, HttpResponse};
use serde_json::json;

use super::models;

pub async fn create_account(body: Json<models::CreateAccountReq>) -> HttpResponse {
  let body = body.into_inner().validate();

  if let Err(err) = body {
    return HttpResponse::BadRequest().json(json!({
      "success": false,
      "message": err["message"],
      "error": err
    }));
  }

  let body = body.unwrap();

  if let Err(err) = body.add_to_db() {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": err["message"],
      "error": err
    }));
  }

  HttpResponse::Ok().json(json!({ "success": true }))
}
