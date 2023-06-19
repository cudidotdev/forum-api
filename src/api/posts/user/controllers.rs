use actix_web::{
  web::{Data, Path},
  HttpResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;

use crate::api::{
  handler_utils::{NoDBClient, NoUserDetails},
  UserAuth,
};

use super::models::{FetchPostsCreatedByUser, FetchPostsSavedByUser};

pub async fn fetch_posts_created_by_user(
  body: Path<FetchPostsCreatedByUser<NoDBClient, NoUserDetails>>,
  user_auth: UserAuth,
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

  let user_details = user_auth.details;

  let body = body.into_inner().add_db_client(&db_client);

  let res = match user_details {
    Some(u) => body.add_user_details(&u).fetch_posts().await,
    None => body.fetch_posts().await,
  };

  match res {
    Ok(data) => HttpResponse::Ok().json(json!({
      "success": false,
      "data": data,
    })),

    Err((s, v)) => HttpResponse::Ok().status(s).json(json!({
      "success": true,
      "message": v["message"],
      "error": {
        "status": s.as_u16(),
        "message": v["message"],
        "name": v["name"]
      }
    })),
  }
}

pub async fn fetch_posts_saved_by_user(
  body: Path<FetchPostsSavedByUser<NoDBClient, NoUserDetails>>,
  user_auth: UserAuth,
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

  let user_details = user_auth.details;

  let body = body.into_inner().add_db_client(&db_client);

  let res = match user_details {
    Some(u) => body.add_user_details(&u).fetch_posts().await,
    None => body.fetch_posts().await,
  };

  match res {
    Ok(data) => HttpResponse::Ok().json(json!({
      "success": true,
      "data": data,
    })),

    Err((s, v)) => HttpResponse::Ok().status(s).json(json!({
      "success": false,
      "message": v["message"],
      "error": {
        "status": s.as_u16(),
        "message": v["message"],
        "name": v["name"]
      }
    })),
  }
}
