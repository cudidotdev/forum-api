mod controllers;
mod models;

use actix_web::web::{self, ServiceConfig};

pub fn view(cfg: &mut ServiceConfig) {
  cfg.route(
    "/trending",
    web::get().to(controllers::get_trending_hashtags),
  );
}
