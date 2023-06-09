use actix_web::web::{self, ServiceConfig};
mod controllers;
mod models;

pub fn view(cfg: &mut ServiceConfig) {
  cfg.route("/save", web::post().to(controllers::save_post));
  cfg.route("/comments", web::get().to(controllers::save_post));
  cfg.route("/comments", web::post().to(controllers::create_comment));
}
