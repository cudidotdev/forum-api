use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use serde_json::json;

use crate::api::UserAuth;

use super::models::SavePost;

pub async fn save_post(
  user_details: UserAuth,
  id: web::Path<i32>,
  db_pool: web::Data<Pool>,
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
  let id = id.into_inner();

  let db_client_res = db_pool.get().await;

  if let Err(e) = db_client_res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e.to_string(),
    }));
  }

  let db_client = db_client_res.unwrap();

  let res = SavePost {
    user_details,
    db_client: &db_client,
    id,
  }
  .run()
  .await;

  match res {
    Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),

    Err(e) => HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e["message"],
      "error": e
    })),
  }
}
