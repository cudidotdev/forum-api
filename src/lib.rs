use actix_web::{
  web::{self, ServiceConfig},
  HttpResponse,
};
use serde_json::json;

mod api;
pub mod middleware;

pub fn app(cfg: &mut ServiceConfig) {
  cfg
    .service(web::scope("/auth").configure(api::auth))
    .service(web::scope("/posts").configure(api::post))
    .default_service(web::to(|| async {
      HttpResponse::NotFound().json(json!({
        "success": false,
        "message": "Route not found. Please check path or method used",
        "error": {
          "status": 404,
          "name": "Route not found",
          "message": "Route not found. Please check path or method used",
        }
      }))
    }));
}
