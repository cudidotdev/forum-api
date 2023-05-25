use actix_web::{
  http::header::{self, HeaderValue},
  App, HttpServer,
};

use actix_cors::Cors;

use serde::{Deserialize, Serialize};

use forum_api::app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv::dotenv().ok();

  let config: Config = config::Config::builder()
    .add_source(config::Environment::default())
    .build()
    .unwrap()
    .try_deserialize()
    .expect("Check env file");

  let server = HttpServer::new(move || {
    App::new()
      .wrap(
        Cors::default()
          .allowed_origin_fn(|origin, _| {
            [
              HeaderValue::from_static("http://localhost:5173"),
              HeaderValue::from_static("http://127.0.0.1:5173"),
            ]
            .contains(origin)
          })
          .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::ORIGIN,
          ])
          .supports_credentials()
          .max_age(3600),
      )
      .configure(app)
  })
  .workers(config.threads.unwrap_or(4))
  .bind(("127.0.0.1", 8080));

  if let Err(e) = server {
    println!("It seems port is already taken. Check info below\n\n{e}");
    return Err(e);
  }

  server.unwrap().run().await
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
  pub threads: Option<usize>,
  pub pg: deadpool_postgres::Config,
}
