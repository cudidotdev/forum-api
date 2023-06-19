use actix_web::web::{self, ServiceConfig};
mod controllers;
mod id;
mod models;
mod user;

pub fn view(cfg: &mut ServiceConfig) {
  cfg
    .route("", web::post().to(controllers::create_post))
    .route("", web::get().to(controllers::fetch_posts))
    .service(web::scope("/{foo:\\d+}").configure(id::view))
    .service(web::scope("/user").configure(user::view));
}
