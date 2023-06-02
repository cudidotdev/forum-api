use actix_web::http::StatusCode;
use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
pub struct CreatePostDetails {
  title: Option<String>,
  topics: Option<Vec<String>>,
  body: Option<String>,
}

pub struct CreatePostDetailsWithDBClient<'a> {
  title: Option<String>,
  topics: Option<Vec<String>>,
  body: Option<String>,
  db_client: &'a Client,
}

impl CreatePostDetails {
  pub fn add_db_client(self, db_client: &Client) -> CreatePostDetailsWithDBClient {
    CreatePostDetailsWithDBClient {
      title: self.title,
      topics: self.topics,
      body: self.body,
      db_client: db_client,
    }
  }
}

impl<'a> CreatePostDetailsWithDBClient<'a> {
  pub fn create_post(&self) -> Result<(), (StatusCode, Value)> {
    Err((
      StatusCode::INTERNAL_SERVER_ERROR,
      json!({"message": "Winner"}),
    ))
  }
}
