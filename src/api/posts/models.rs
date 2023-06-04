use std::marker::PhantomData;

use crate::api::UserAuthDetails;
use actix_web::http::StatusCode;
use chrono::Utc;
use deadpool_postgres::Client;
use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_postgres::Statement;

#[derive(Serialize, Deserialize)]
pub struct CreatePostDetails<D, U, V> {
  title: String,
  topics: Vec<String>,
  body: String,
  #[serde(skip_deserializing)]
  db_client: D,
  #[serde(skip_deserializing)]
  user_details: U,
  #[serde(skip_deserializing)]
  validated: PhantomData<V>,
}
#[derive(Default)]
pub struct NoDBClient;
pub struct DBClient<'a>(&'a Client);
#[derive(Default)]
pub struct NoUserDetails;
pub struct UserDetails<'a>(&'a UserAuthDetails);
#[derive(Default)]
pub struct NotValidated;
pub struct Validated;

impl<U, V> CreatePostDetails<NoDBClient, U, V> {
  pub fn add_db_client(self, db_client: &Client) -> CreatePostDetails<DBClient, U, V> {
    CreatePostDetails {
      title: self.title,
      topics: self.topics,
      body: self.body,
      db_client: DBClient(db_client),
      user_details: self.user_details,
      validated: PhantomData,
    }
  }
}

impl<D, V> CreatePostDetails<D, NoUserDetails, V> {
  pub fn add_user_details(
    self,
    user_details: &UserAuthDetails,
  ) -> CreatePostDetails<D, UserDetails, V> {
    CreatePostDetails {
      title: self.title,
      topics: self.topics,
      body: self.body,
      db_client: self.db_client,
      user_details: UserDetails(user_details),
      validated: PhantomData,
    }
  }
}

impl<'a> CreatePostDetails<DBClient<'a>, UserDetails<'a>, NotValidated> {
  pub fn validate(
    mut self,
  ) -> Result<CreatePostDetails<DBClient<'a>, UserDetails<'a>, Validated>, (StatusCode, Value)> {
    self.title = self.title.trim().to_owned();
    self.body = self.body.trim().to_owned();

    if self.title.len() > 100 {
      return Err((
        StatusCode::BAD_REQUEST,
        json!({"message": "Title should not have more 100 characters"}),
      ));
    }

    if self.body.len() > 1000 {
      return Err((
        StatusCode::BAD_REQUEST,
        json!({"message": "Title should not have more 1000 characters"}),
      ));
    }

    lazy_static! {
      static ref RE: Result<Regex, regex::Error> = Regex::new(r"[^A-Za-z\s]+");
    }

    let mut all_under_51 = true;

    self.topics.iter_mut().for_each(|s| {
      *s = s.trim().to_owned();
      s.make_ascii_lowercase();
      s.get_mut(0..1).map(|a| a.make_ascii_uppercase());

      if RE.is_ok() {
        *s = String::from(RE.as_ref().unwrap().replace_all(s, "").to_string());
      }

      all_under_51 = all_under_51 && s.len() <= 50;
    });

    if !all_under_51 {
      return Err((
        StatusCode::BAD_REQUEST,
        json!({"message": "Topic names should not be more than 50 characters"}),
      ));
    }

    Ok(CreatePostDetails {
      title: self.title,
      topics: self.topics,
      body: self.body,
      db_client: self.db_client,
      user_details: self.user_details,
      validated: PhantomData,
    })
  }
}

impl<'a, U, V> CreatePostDetails<DBClient<'a>, U, V> {
  fn get_db_client(&self) -> &'a Client {
    self.db_client.0
  }

  async fn get_create_post_statment(&self) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "INSERT INTO posts(title, body, created_at) VALUES ($1, $2, $3) RETURNING id";
    self.get_db_client().prepare(stmt).await
  }

  async fn get_upsert_topic_statement(&self) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "INSERT INTO topics(name, color, created_at)
                      VALUES ($1, $2, $3)
                      ON CONFLICT (name) DO UPDATE
                      SET name = EXCLUDED.name
                      RETURNING id";

    self.get_db_client().prepare(stmt).await
  }
}

impl<'a> CreatePostDetails<DBClient<'a>, UserDetails<'a>, Validated> {
  pub async fn create_post(&self) -> Result<i32, (StatusCode, Value)> {
    let stmt = self.get_create_post_statment().await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "message": e.to_string() }),
      )
    })?;

    let post_id = self
      .get_db_client()
      .query(&stmt, &[&self.title, &self.body, &Utc::now().naive_utc()])
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({ "message": e.to_string() }),
        )
      })?
      .get(0)
      .ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "message": "No id returned" }),
      ))?
      .try_get("id")
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({ "message": e.to_string() }),
        )
      })?;

    Ok(post_id)
  }

  fn get_random_color(&self) -> &str {
    let a = rand::thread_rng().gen_range(0..6);

    match a {
      0 => "green",
      1 => "blue",
      2 => "red",
      3 => "yellow",
      4 => "purple",
      5 => "violet",
      _ => "red",
    }
  }
}
