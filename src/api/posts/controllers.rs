use actix_web::{
  web::{self, Query},
  HttpResponse,
};
use deadpool_postgres::{Client, Pool};
use serde_json::json;

use super::models;

use crate::api::{
  handler_utils::{NoDBClient, NoUserDetails, NotValidated},
  UserAuth,
};

pub async fn create_post(
  user_details: UserAuth,
  db_pool: web::Data<Pool>,
  body: web::Json<models::CreatePostDetails<NoDBClient, NoUserDetails, NotValidated>>,
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

  let db_client: Client = db_client_res.unwrap();

  let res = body
    .into_inner()
    .add_db_client(&db_client)
    .add_user_details(&user_details)
    .validate();

  let res = match res {
    Ok(p) => p.create_post().await,
    Err(e) => Err(e),
  };

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

pub async fn fetch_posts(
  db_pool: web::Data<Pool>,
  user_details: UserAuth,
  query: Query<models::FetchPosts<NoDBClient, NoUserDetails, NotValidated>>,
) -> HttpResponse {
  let user_details = user_details.details;

  let db_client_res = db_pool.get().await;

  if let Err(e) = db_client_res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e.to_string(),
    }));
  }

  let db_client: Client = db_client_res.unwrap();

  let query = query.into_inner().add_db_client(&db_client).validate();

  if let Err((s, v)) = query {
    return HttpResponse::Ok().status(s).json(json!({
      "success": false,
      "message": v["message"],
      "error":v
    }));
  }

  let query = query.unwrap();

  let res = if let Some(u) = user_details {
    query.add_user_details(&u).fetch_posts().await
  } else {
    query.fetch_posts().await
  };

  match res {
    Ok(v) => HttpResponse::Ok().json(json!({
      "success": true,
      "data": v
    })),

    Err((s, v)) => HttpResponse::Ok().status(s).json(json!({
      "success": false,
      "message": v["message"],
      "error": v
    })),
  }
}
