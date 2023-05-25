use actix_web::web::{self, ServiceConfig};

pub fn app(cfg: &mut ServiceConfig) {
  cfg.route("/", web::get().to(|| async { "working" }));
}
