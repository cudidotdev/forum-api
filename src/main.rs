use actix_web::{
  error,
  http::{
    header::{self, HeaderValue},
    Method,
  },
  web, App, HttpResponse, HttpServer,
};

use actix_cors::Cors;

use deadpool_postgres::Runtime;
use serde::{Deserialize, Serialize};

use forum_api::app;
use serde_json::json;
use tokio_postgres::NoTls;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv::dotenv().ok();

  let config: Config = config::Config::builder()
    .add_source(config::Environment::default())
    .build()
    .unwrap()
    .try_deserialize()
    .expect("Check env file");

  let pool_res = config.pg.create_pool(Some(Runtime::Tokio1), NoTls);

  if let Err(e) = pool_res {
    eprintln!("Postgres pool creation error\n\n {e:#?}");
    return Ok(());
  }

  let pool = pool_res.unwrap();

  let server = HttpServer::new(move || {
    let json_config = web::JsonConfig::default()
      .limit(4096)
      .error_handler(|err, _req| {
        error::InternalError::from_response(
          err,
          HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Invalid data in request body"
          })),
        )
        .into()
      });

    App::new()
      .app_data(json_config)
      .app_data(web::Data::new(pool.clone()))
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
          .allow_any_method()
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

  println!("Server started at port 8080");

  server.unwrap().run().await
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
  pub threads: Option<usize>,
  pub pg: deadpool_postgres::Config,
}
