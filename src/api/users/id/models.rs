use actix_web::http::StatusCode;
use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_postgres::{Row, Statement};

use crate::api::{
  handler_utils::{NoDBClient, NoUserDetails, WithDBClient, WithUserDetails},
  posts::FetchPostsResponse,
  UserAuthDetails,
};

#[derive(Deserialize)]
pub struct FetchUserDetails<D> {
  user_id: i32,
  #[serde(skip_deserializing)]
  db_client: D,
}

#[derive(Serialize)]
pub struct UserDetails {
  id: i32,
  username: String,
}

impl<'a> FetchUserDetails<NoDBClient> {
  pub fn add_db_client(self, db_client: &'a Client) -> FetchUserDetails<WithDBClient<'a>> {
    FetchUserDetails {
      user_id: self.user_id,
      db_client: WithDBClient(db_client),
    }
  }
}

impl<'a> FetchUserDetails<WithDBClient<'a>> {
  pub async fn fetch(&self) -> Result<UserDetails, (StatusCode, Value)> {
    self
      .get_db_client()
      .query(&self.get_select_statement().await?, &[&self.user_id])
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })?
      .into_iter()
      .map(|r| UserDetails::from_row(&r))
      .nth(0)
      .ok_or((
        StatusCode::NOT_FOUND,
        json!({ "message": format!("No user found with id {}", self.user_id) }),
      ))?
  }

  async fn get_select_statement(&self) -> Result<Statement, (StatusCode, Value)> {
    let stmt = "SELECT id, username FROM users WHERE id = $1";

    self.get_db_client().prepare(stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }

  fn get_db_client(&self) -> &'a Client {
    self.db_client.0
  }
}

impl UserDetails {
  pub fn from_row(row: &Row) -> Result<UserDetails, (StatusCode, Value)> {
    let id = row.try_get::<&str, i32>("id");
    let username = row.try_get::<&str, String>("username");

    match (id, username) {
      (Ok(id), Ok(username)) => Ok(UserDetails { id, username }),
      _ => Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message":"Error converting postgres types" }),
      )),
    }
  }
}

#[derive(Deserialize)]
pub struct FetchPostsCreatedByUser<D, U> {
  user_id: i32,
  #[serde(skip_deserializing)]
  db_client: D,
  #[serde(skip_deserializing)]
  user_details: U,
}

impl<'a, U> FetchPostsCreatedByUser<NoDBClient, U> {
  pub fn add_db_client(
    self,
    db_client: &'a Client,
  ) -> FetchPostsCreatedByUser<WithDBClient<'a>, U> {
    FetchPostsCreatedByUser {
      user_id: self.user_id,
      db_client: WithDBClient(db_client),
      user_details: self.user_details,
    }
  }
}

impl<'a, D> FetchPostsCreatedByUser<D, NoUserDetails> {
  pub fn add_user_details(
    self,
    user_details: &'a UserAuthDetails,
  ) -> FetchPostsCreatedByUser<D, WithUserDetails<'a>> {
    FetchPostsCreatedByUser {
      user_id: self.user_id,
      db_client: self.db_client,
      user_details: WithUserDetails(user_details),
    }
  }
}

impl<'a, D> FetchPostsCreatedByUser<D, WithUserDetails<'a>> {
  fn get_user_details(&self) -> &'a UserAuthDetails {
    self.user_details.0
  }
}

impl<'a, U> FetchPostsCreatedByUser<WithDBClient<'a>, U> {
  fn get_db_client(&self) -> &'a Client {
    self.db_client.0
  }
}

impl<'a> FetchPostsCreatedByUser<WithDBClient<'a>, NoUserDetails> {
  pub async fn fetch_posts(&self) -> Result<Vec<FetchPostsResponse>, (StatusCode, Value)> {
    self
      .get_db_client()
      .query(&self.get_select_statement().await?, &[&self.user_id])
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })?
      .into_iter()
      .map(|r| FetchPostsResponse::from_row(&r))
      .collect::<Result<Vec<_>, _>>()
  }

  async fn get_select_statement(&self) -> Result<Statement, (StatusCode, Value)> {
    let stmt ="SELECT p.id, p.title, p.body, u.id author_id, u.username author_name, 
     p.created_at, ARRAY_AGG(DISTINCT t.name ||':'|| t.color::TEXT) hashtags, COUNT(DISTINCT c.*) comments, COUNT(DISTINCT s.*) saves FROM posts p 
     INNER JOIN posts_hashtags_relationship r ON p.id = r.post_id 
     INNER JOIN hashtags t ON t.id = r.hashtag_id
     INNER JOIN users u ON u.id = p.user_id
     LEFT JOIN post_comments c ON p.id = c.post_id
     LEFT JOIN saved_posts s ON s.post_id = p.id
     WHERE u.id = $1
     GROUP BY p.id, u.id
     ORDER BY created_at DESC";

    self.get_db_client().prepare(stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }
}

impl<'a> FetchPostsCreatedByUser<WithDBClient<'a>, WithUserDetails<'a>> {
  pub async fn fetch_posts(&self) -> Result<Vec<FetchPostsResponse>, (StatusCode, Value)> {
    self
      .get_db_client()
      .query(
        &self.get_select_statement().await?,
        &[&self.user_id, &self.get_user_details().id],
      )
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })?
      .into_iter()
      .map(|r| FetchPostsResponse::from_row(&r))
      .collect::<Result<Vec<_>, _>>()
  }

  async fn get_select_statement(&self) -> Result<Statement, (StatusCode, Value)> {
    let stmt = "SELECT p.id, p.title, p.body, u.id author_id, u.username author_name,
      (s.post_id IS NOT NULL) saved, p.created_at, ARRAY_AGG(DISTINCT t.name ||':'|| t.color::TEXT) hashtags, COUNT(DISTINCT c.*) comments, COUNT(DISTINCT ss.*) saves FROM posts p
      INNER JOIN  posts_hashtags_relationship r ON p.id = r.post_id
      INNER JOIN hashtags t ON t.id = r.hashtag_id
      INNER JOIN users u ON u.id = p.user_id
      LEFT JOIN saved_posts s ON s.post_id = p.id AND s.user_id = $2
      LEFT JOIN saved_posts ss ON ss.post_id = p.id
      LEFT JOIN post_comments c ON p.id = c.post_id
      WHERE u.id = $1 
      GROUP BY p.id, u.id, s.post_id
      ORDER BY created_at DESC";

    self.get_db_client().prepare(stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }
}

#[derive(Deserialize)]
pub struct FetchPostsSavedByUser<D, U> {
  user_id: i32,
  #[serde(skip_deserializing)]
  db_client: D,
  #[serde(skip_deserializing)]
  user_details: U,
}

impl<'a, U> FetchPostsSavedByUser<NoDBClient, U> {
  pub fn add_db_client(self, db_client: &'a Client) -> FetchPostsSavedByUser<WithDBClient<'a>, U> {
    FetchPostsSavedByUser {
      user_id: self.user_id,
      db_client: WithDBClient(db_client),
      user_details: self.user_details,
    }
  }
}

impl<'a, D> FetchPostsSavedByUser<D, NoUserDetails> {
  pub fn add_user_details(
    self,
    user_details: &'a UserAuthDetails,
  ) -> FetchPostsSavedByUser<D, WithUserDetails<'a>> {
    FetchPostsSavedByUser {
      user_id: self.user_id,
      db_client: self.db_client,
      user_details: WithUserDetails(user_details),
    }
  }
}

impl<'a, D> FetchPostsSavedByUser<D, WithUserDetails<'a>> {
  fn get_user_details(&self) -> &'a UserAuthDetails {
    self.user_details.0
  }
}

impl<'a, U> FetchPostsSavedByUser<WithDBClient<'a>, U> {
  fn get_db_client(&self) -> &'a Client {
    self.db_client.0
  }
}

impl<'a> FetchPostsSavedByUser<WithDBClient<'a>, NoUserDetails> {
  pub async fn fetch_posts(&self) -> Result<Vec<FetchPostsResponse>, (StatusCode, Value)> {
    self
      .get_db_client()
      .query(&self.get_select_statement().await?, &[&self.user_id])
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })?
      .into_iter()
      .map(|r| FetchPostsResponse::from_row(&r))
      .collect::<Result<Vec<_>, _>>()
  }

  async fn get_select_statement(&self) -> Result<Statement, (StatusCode, Value)> {
    let stmt = "SELECT p.id, p.title, p.body, u.id author_id, u.username author_name, 
     p.created_at, ARRAY_AGG(DISTINCT t.name ||':'|| t.color::TEXT) hashtags, COUNT(DISTINCT c.*) comments, 0::BIGINT saves FROM posts p 
     INNER JOIN posts_hashtags_relationship r ON p.id = r.post_id 
     INNER JOIN hashtags t ON t.id = r.hashtag_id
     INNER JOIN users u ON u.id = p.user_id
     LEFT JOIN post_comments c ON p.id = c.post_id
     LEFT JOIN saved_posts s ON s.post_id = p.id
     WHERE s.user_id = $1
     GROUP BY p.id, u.id
     ORDER BY created_at DESC";

    self.get_db_client().prepare(stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }
}

impl<'a> FetchPostsSavedByUser<WithDBClient<'a>, WithUserDetails<'a>> {
  pub async fn fetch_posts(&self) -> Result<Vec<FetchPostsResponse>, (StatusCode, Value)> {
    self
      .get_db_client()
      .query(
        &self.get_select_statement().await?,
        &[&self.user_id, &self.get_user_details().id],
      )
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })?
      .into_iter()
      .map(|r| FetchPostsResponse::from_row(&r))
      .collect::<Result<Vec<_>, _>>()
  }

  async fn get_select_statement(&self) -> Result<Statement, (StatusCode, Value)> {
    let stmt = "SELECT p.id, p.title, p.body, u.id author_id, u.username author_name,
      (s.post_id IS NOT NULL) saved, p.created_at, ARRAY_AGG(DISTINCT t.name ||':'|| t.color::TEXT) hashtags, COUNT(DISTINCT c.*) comments, 0::BIGINT saves FROM posts p
      INNER JOIN  posts_hashtags_relationship r ON p.id = r.post_id
      INNER JOIN hashtags t ON t.id = r.hashtag_id
      INNER JOIN users u ON u.id = p.user_id
      LEFT JOIN saved_posts s ON s.post_id = p.id AND s.user_id = $2
      LEFT JOIN saved_posts ss ON ss.post_id = p.id
      LEFT JOIN post_comments c ON p.id = c.post_id
      WHERE ss.user_id = $1
      GROUP BY p.id, u.id, s.post_id
      ORDER BY created_at DESC";

    self.get_db_client().prepare(stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }
}
