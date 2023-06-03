use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use serde_json::json;

use super::models::{self, NoDBClient, NoUserDetails, NotFormated};
use crate::api::UserAuth;

pub async fn create_post(
  user_details: UserAuth,
  db_pool: web::Data<Pool>,
  body: web::Json<models::CreatePostDetails<NoDBClient, NoUserDetails, NotFormated>>,
) -> HttpResponse {
  if user_details.details.is_none() {
    return HttpResponse::Forbidden().json(json!({
      "success": false,
      "message": "User not signed in",
      "error": {
        "name": "re-auth",
        "message": "User not signed in"
      }
    }));
  };

  let user_details = user_details.details.unwrap();

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
    .format()
    .add_db_client(&db_client)
    .add_user_details(&user_details)
    .create_post()
    .await;

  match res {
    Ok(id) => HttpResponse::Ok().json(json!({
      "success": true,
      "data": {
        "id": id
      }
    })),

    Err((s, v)) => HttpResponse::Ok().status(s).json(json!({
      "success": false,
      "message": v["message"],
      "error": v
    })),
  }
}
