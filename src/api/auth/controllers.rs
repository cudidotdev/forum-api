use actix_web::{
  web::{Data, Json},
  HttpResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;

use super::models::{self, UserAuth};

pub async fn verify(user_detail: UserAuth) -> HttpResponse {
  user_detail.details.map_or(
    HttpResponse::Ok().json(json!({
       "success": false,
       "data": null
    })),
    |r| {
      HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
          "id": r.id,
          "username": r.username,
        }
      }))
    },
  )
}

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

  let res = res.unwrap();

  HttpResponse::Ok().json(json!({
      "success": true,
      "data": {
        "id": res.id,
        "username": res.username,
        "access_token": res.to_jwt()
  }}))
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

  let res = body.into_inner().add_db_client(&db_client).validate().await;

  if let Err(err) = res {
    return HttpResponse::BadRequest().json(json!({
      "success": false,
      "message": err["message"],
      "error": err
    }));
  }

  let res = res.unwrap();

  HttpResponse::Ok().json(json!({
      "success": true,
      "data": {
        "id": res.id,
        "username": res.username,
        "access_token": res.to_jwt()
  }}))
}
