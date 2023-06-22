mod controllers;
mod id;
mod models;

use actix_web::web::{self, ServiceConfig};

pub fn view(cfg: &mut ServiceConfig) {
  cfg.service(web::scope("{user_id}").configure(id::view));
}
