use actix_web::web::{self, ServiceConfig};

mod controllers;
mod models;

pub fn view(cfg: &mut ServiceConfig) {
  cfg
    .route(
      "/{user_id}",
      web::get().to(controllers::fetch_posts_created_by_user),
    )
    .route(
      "/{user_id}/saves",
      web::get().to(controllers::fetch_posts_saved_by_user),
    );
}
