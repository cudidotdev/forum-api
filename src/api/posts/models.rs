use std::marker::PhantomData;

use crate::api::UserAuthDetails;
use actix_web::http::StatusCode;
use deadpool_postgres::Client;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
pub struct CreatePostDetails<D, U, F> {
  title: String,
  topics: Vec<String>,
  body: String,
  #[serde(skip_deserializing)]
  db_client: D,
  #[serde(skip_deserializing)]
  user_details: U,
  #[serde(skip_deserializing)]
  formatted: PhantomData<F>,
}
#[derive(Default)]
pub struct NoDBClient;
pub struct DBClient<'a>(&'a Client);
#[derive(Default)]
pub struct NoUserDetails;
pub struct UserDetails<'a>(&'a UserAuthDetails);
#[derive(Default)]
pub struct NotFormated;
pub struct Formatted;

impl<U, F> CreatePostDetails<NoDBClient, U, F> {
  pub fn add_db_client(self, db_client: &Client) -> CreatePostDetails<DBClient, U, F> {
    CreatePostDetails {
      title: self.title,
      topics: self.topics,
      body: self.body,
      db_client: DBClient(db_client),
      user_details: self.user_details,
      formatted: PhantomData,
    }
  }
}

impl<D, F> CreatePostDetails<D, NoUserDetails, F> {
  pub fn add_user_details(
    self,
    user_details: &UserAuthDetails,
  ) -> CreatePostDetails<D, UserDetails, F> {
    CreatePostDetails {
      title: self.title,
      topics: self.topics,
      body: self.body,
      db_client: self.db_client,
      user_details: UserDetails(user_details),
      formatted: PhantomData,
    }
  }
}

impl<D, U> CreatePostDetails<D, U, NotFormated> {
  pub fn format(mut self) -> CreatePostDetails<D, U, Formatted> {
    lazy_static! {
      static ref RE: Result<Regex, regex::Error> = Regex::new("[^A-Za-z]+");
    }

    self.topics.iter_mut().for_each(|s| {
      s.make_ascii_lowercase();
      s.get_mut(0..1).map(|a| a.make_ascii_uppercase());

      if RE.is_ok() {
        *s = String::from(RE.as_ref().unwrap().replace_all(s, "").to_string());
      }
    });

    CreatePostDetails {
      title: self.title,
      topics: self.topics,
      body: self.body,
      db_client: self.db_client,
      user_details: self.user_details,
      formatted: PhantomData,
    }
  }
}

impl<'a> CreatePostDetails<DBClient<'a>, UserDetails<'a>, Formatted> {
  pub async fn create_post(&self) -> Result<i32, (StatusCode, Value)> {
    Err((
      StatusCode::INTERNAL_SERVER_ERROR,
      json!({ "message": "won" }),
    ))
  }
}
