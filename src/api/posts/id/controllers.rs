use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use serde_json::json;

use crate::api::{
  handler_utils::{NoDBClient, NoUserDetails, NotValidated},
  UserAuth,
};

use super::models::{CreateComment, FetchComments, FetchPost, SavePost};

pub async fn fetch_post(id: web::Path<i32>, db_pool: web::Data<Pool>) -> HttpResponse {
  let id = id.into_inner();

  let db_client_res = db_pool.get().await;

  if let Err(e) = db_client_res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e.to_string(),
    }));
  }

  let db_client = db_client_res.unwrap();

  let res = FetchPost {
    db_client: &db_client,
    id,
  }
  .exec()
  .await;

  match res {
    Ok(post) => HttpResponse::Ok().json(json!({
      "success": true,
      "data": post
    })),

    Err((s, e)) => HttpResponse::Ok().status(s).json(json!({
      "success": false,
      "message": e["message"],
      "error": e
    })),
  }
}

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
  .exec()
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

pub async fn unsave_post(
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
  .exec_reverse()
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

pub async fn create_comment(
  user_details: UserAuth,
  post_id: web::Path<i32>,
  body: web::Json<CreateComment<NoDBClient, NoUserDetails, NotValidated>>,
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
  let post_id = post_id.into_inner();

  let db_client_res = db_pool.get().await;

  if let Err(e) = db_client_res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e.to_string(),
    }));
  }

  let db_client = db_client_res.unwrap();

  let body = body
    .into_inner()
    .add_details(post_id, &db_client, &user_details)
    .validate()
    .await;

  if let Err(e) = body {
    return HttpResponse::BadRequest().json(json!({
      "success": false,
      "message": e["message"],
      "error": e
    }));
  }

  match body.unwrap().exec().await {
    Ok(id) => HttpResponse::Ok().json(json!({
      "success": true,
      "data": {
        "id": id
      }
    })),

    Err(v) => HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": v["message"],
      "error": v
    })),
  }
}

pub async fn fetch_comments(
  post_id: web::Path<i32>,
  query: web::Query<FetchComments<NoDBClient, NotValidated>>,
  db_pool: web::Data<Pool>,
) -> HttpResponse {
  let db_client_res = db_pool.get().await;

  if let Err(e) = db_client_res {
    return HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": e.to_string(),
    }));
  }

  let db_client = db_client_res.unwrap();

  let post_id = post_id.into_inner();

  let query = query
    .into_inner()
    .add_details(&db_client, post_id)
    .validate();

  if let Err(v) = query {
    return HttpResponse::BadRequest().json(json!({
      "success": false,
      "message": v["message"],
      "error":v
    }));
  }

  let res = query.unwrap().fetch_comments().await;

  match res {
    Ok(d) => HttpResponse::Ok().json(json!({
      "success": true,
      "data": d
    })),

    Err(v) => HttpResponse::InternalServerError().json(json!({
      "success": false,
      "message": v["message"],
      "error":v
    })),
  }
}
