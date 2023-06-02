use actix_web::web::{self, ServiceConfig};
mod controllers;
pub mod models;

pub fn view(cfg: &mut ServiceConfig) {
  cfg.route("", web::get().to(controllers::verify));
  cfg.route("/sign-in", web::post().to(controllers::login));
  cfg.route("/sign-up", web::post().to(controllers::create_account));
}
