use std::marker::PhantomData;

use deadpool_postgres::Client;
use futures_util::sink::With;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_postgres::Statement;

use crate::api::{
  handler_utils::{
    NoDBClient, NoUserDetails, NotValidated, Validated, WithDBClient, WithUserDetails,
  },
  UserAuthDetails,
};

pub struct SavePost<'a> {
  pub user_details: UserAuthDetails,
  pub db_client: &'a Client,
  pub id: i32,
}

impl<'a> SavePost<'a> {
  pub async fn run(&self) -> Result<(), Value> {
    self
      .db_client
      .query(
        &self.get_insert_statement().await?,
        &[&self.user_details.id, &self.id],
      )
      .await
      .map_err(|e| json!({"message": e.to_string()}))
      .map(|_| ())
  }

  pub async fn get_insert_statement(&self) -> Result<Statement, Value> {
    let stmt = "INSERT INTO saved_posts (user_id, post_id) VALUES ($1, $2)";

    self
      .db_client
      .prepare(stmt)
      .await
      .map_err(|e| json!({ "message":e.to_string()  }))
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateComment<D, U, V> {
  #[serde(skip_deserializing)]
  post_id: i32,

  body: String,

  comment_id: Option<i32>,

  #[serde(skip_deserializing)]
  db_client: D,

  #[serde(skip_deserializing)]
  user_details: U,

  #[serde(skip_deserializing)]
  validated: PhantomData<V>,
}

impl<'a, V> CreateComment<NoDBClient, NoUserDetails, V> {
  pub fn add_details(
    self,
    post_id: i32,
    db_client: &'a Client,
    user_details: &'a UserAuthDetails,
  ) -> CreateComment<WithDBClient<'a>, WithUserDetails<'a>, V> {
    CreateComment {
      post_id,
      body: self.body,
      comment_id: self.comment_id,
      db_client: WithDBClient(db_client),
      user_details: WithUserDetails(user_details),
      validated: PhantomData,
    }
  }
}

impl<'a, D, V> CreateComment<D, WithUserDetails<'a>, V> {
  fn get_user_details(&self) -> &'a UserAuthDetails {
    self.user_details.0
  }
}

impl<'a, U, V> CreateComment<WithDBClient<'a>, U, V> {
  fn get_db_client(&self) -> &'a Client {
    self.db_client.0
  }
}

impl<'a, U> CreateComment<WithDBClient<'a>, U, NotValidated> {
  pub async fn validate(mut self) -> Result<CreateComment<WithDBClient<'a>, U, Validated>, Value> {
    self.body = self.body.trim().to_owned();

    if self.body.len() > 500 {
      return Err(json!({
        "name": "body",
        "message": "Comment should be less than 500 characters"
      }));
    }

    let is_comment_under_post = self.is_comment_under_post().await?;

    if !is_comment_under_post {
      return Err(json!({"message": "Comment does not exists in post"}));
    }

    Ok(CreateComment {
      post_id: self.post_id,
      comment_id: self.comment_id,
      body: self.body,
      db_client: self.db_client,
      user_details: self.user_details,
      validated: PhantomData,
    })
  }

  async fn is_comment_under_post(&self) -> Result<bool, Value> {
    if let None = self.comment_id {
      return Ok(true);
    }

    let stmt = "SELECT EXISTS (SELECT 1 FROM post_comments WHERE post_id = $1 AND id = $2) exists";

    let stmt = self
      .get_db_client()
      .prepare(stmt)
      .await
      .map_err(|e| json!({"message": e.to_string()}))?;

    self
      .get_db_client()
      .query(&stmt, &[&self.post_id, &self.comment_id])
      .await
      .map_err(|e| json!({"message": e.to_string()}))?
      .get(0)
      .ok_or(json!({"message": "No response from db"}))?
      .try_get("exists")
      .map_err(|e| json!({"message": e.to_string()}))
  }
}

impl<'a, U> CreateComment<WithDBClient<'a>, U, Validated> {
  async fn get_insert_statement(&self) -> Result<Statement, Value> {
    let stmt = "INSERT INTO post_comments (post_id, user_id, comment_id, body)
      VALUES ($1, $2, $3, $4) RETURNING id";

    self
      .get_db_client()
      .prepare(stmt)
      .await
      .map_err(|e| json!({"message": e.to_string()}))
  }
}

impl<'a> CreateComment<WithDBClient<'a>, WithUserDetails<'a>, Validated> {
  pub async fn exec(&self) -> Result<i32, Value> {
    self
      .get_db_client()
      .query(
        &self.get_insert_statement().await?,
        &[
          &self.post_id,
          &self.get_user_details().id,
          &self.comment_id,
          &self.body,
        ],
      )
      .await
      .map_err(|e| json!({"message": e.to_string()}))?
      .get(0)
      .ok_or(json!({"message": "No response from db"}))?
      .try_get("id")
      .map_err(|e| json!({"message": e.to_string()}))
  }
}
