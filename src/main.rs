use actix_web::{
  dev::{Service, ServiceResponse},
  http::header::{self, HeaderValue},
  web, App, HttpServer,
};

use futures_util::future::FutureExt;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv::dotenv().ok();

  let config = config::Config::builder()
    .add_source(config::Environment::default())
    .build()
    .unwrap();

  let server = HttpServer::new(move || {
    App::new()
      .wrap_fn(|req, srv| {
        let origin = req
          .headers()
          .get("origin")
          .unwrap_or(&HeaderValue::from_static(""))
          .to_owned();

        srv
          .call(req)
          .map(|res: Result<ServiceResponse, actix_web::Error>| {
            if res.is_err() {
              return res;
            }

            let allow_headers =
              HeaderValue::from_static("Content-Type, Authorization, Accept, Origin");
            let allow_methods = HeaderValue::from_static("GET, POST, PUT, PATCH, DELETE");
            let allow_credentials = HeaderValue::from_static("true");

            let mut res = res.unwrap();
            let headers = res.headers_mut();

            if [
              HeaderValue::from_static("http://localhost:5173"),
              HeaderValue::from_static("http://127.0.0.1:5173"),
            ]
            .contains(&origin)
            {
              headers.append(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
              headers.append(header::ACCESS_CONTROL_ALLOW_HEADERS, allow_headers);
              headers.append(header::ACCESS_CONTROL_ALLOW_METHODS, allow_methods);
              headers.append(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials)
            }

            Ok(res)
          })
      })
      .route("/", web::get().to(|| async { "working" }))
  })
  .bind(("127.0.0.1", 8080));

  if let Err(e) = server {
    println!("It seems port is already taken. Check info below\n\n{e}");
    return Err(e);
  }

  server.unwrap().run().await
}
