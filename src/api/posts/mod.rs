use actix_web::web::{self, ServiceConfig};
mod controllers;
mod models;

pub fn view(cfg: &mut ServiceConfig) {
  cfg.route("", web::post().to(controllers::create_post));
}
