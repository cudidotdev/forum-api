mod controllers;
mod models;

use actix_web::web::{self, ServiceConfig};

pub fn view(cfg: &mut ServiceConfig) {
  cfg.route("", web::get().to(controllers::fetch_user));
  cfg.route(
    "/posts",
    web::get().to(controllers::fetch_posts_created_by_user),
  );
  cfg.route(
    "/saves",
    web::get().to(controllers::fetch_posts_saved_by_user),
  );
}
