use std::future::{ready, Ready};

use actix_web::{
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  http::header,
  Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;

use crate::api::UserAuthDetails;

pub struct Authenticate;
pub struct AuthenticateMiddleware<S> {
  service: S,
}

impl<S, B> Transform<S, ServiceRequest> for Authenticate
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Transform = AuthenticateMiddleware<S>;
  type InitError = ();
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(AuthenticateMiddleware { service }))
  }
}

impl<S, B> Service<ServiceRequest> for AuthenticateMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let user_details = req
      .headers()
      .get(header::AUTHORIZATION)
      .ok_or(())
      .and_then(|h| h.to_str().map_err(|_| ()))
      .ok()
      .and_then(|s| s.split_whitespace().nth(1))
      .ok_or(())
      .and_then(|t| UserAuthDetails::from_jwt(t).map_err(|_| ()))
      .ok();

    if let Some(u) = user_details {
      req.extensions_mut().insert(u);
    }

    let fut = self.service.call(req);

    Box::pin(async move {
      let res = fut.await?;
      Ok(res)
    })
  }
}
