use std::marker::PhantomData;

use actix_web::http::StatusCode;
use chrono::{NaiveDateTime, Utc};
use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_postgres::{Row, Statement};

use crate::api::{
  handler_utils::{
    NoDBClient, NoUserDetails, NotValidated, Validated, WithDBClient, WithUserDetails,
  },
  posts::models::FetchPostsResponse,
  UserAuthDetails,
};

pub struct FetchPost<'a> {
  pub db_client: &'a Client,
  pub user_id: Option<i32>,
  pub id: i32,
}

impl<'a> FetchPost<'a> {
  pub async fn exec(&self) -> Result<FetchPostsResponse, (StatusCode, Value)> {
    self
      .db_client
      .query(
        &self.get_select_statement().await?,
        &[&self.id, &self.user_id.unwrap_or_default()],
      )
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })?
      .get(0)
      .ok_or((
        StatusCode::NOT_FOUND,
        json!({"message": "No post found with such id"}),
      ))
      .map(|r| FetchPostsResponse::from_row(r))?
  }

  async fn get_select_statement(&self) -> Result<Statement, (StatusCode, Value)> {
    let stmt = "SELECT p.id, p.title, p.body, u.id author_id, u.username author_name, 
      (s.post_id IS NOT NULL) saved, p.created_at, ARRAY_AGG(DISTINCT t.name||':'||t.color) hashtags, COUNT(DISTINCT c.*) comments, COUNT(DISTINCT ss.*) saves FROM posts p 
      INNER JOIN  posts_hashtags_relationship r ON p.id = r.post_id 
      INNER JOIN hashtags t ON t.id = r.hashtag_id 
      INNER JOIN users u ON u.id = p.user_id 
      LEFT JOIN saved_posts s ON s.post_id = p.id AND s.user_id = $2 
      LEFT JOIN saved_posts ss ON ss.post_id = p.id
      LEFT JOIN post_comments c ON p.id = c.post_id
      WHERE p.id = $1
      GROUP BY p.id, u.id, s.post_id".to_owned();

    self.db_client.prepare(&stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }
}

pub struct SavePost<'a> {
  pub user_details: UserAuthDetails,
  pub db_client: &'a Client,
  pub id: i32,
}

impl<'a> SavePost<'a> {
  pub async fn exec(&self) -> Result<(), Value> {
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

  pub async fn exec_reverse(&self) -> Result<(), Value> {
    self
      .db_client
      .query(
        &self.get_delete_statement().await?,
        &[&self.user_details.id, &self.id],
      )
      .await
      .map_err(|e| json!({"message": e.to_string()}))
      .map(|_| ())
  }

  pub async fn get_insert_statement(&self) -> Result<Statement, Value> {
    let stmt = "INSERT INTO saved_posts (user_id, post_id) VALUES ($1, $2)
      ON CONFLICT (user_id, post_id) DO NOTHING";

    self
      .db_client
      .prepare(stmt)
      .await
      .map_err(|e| json!({ "message":e.to_string()  }))
  }

  pub async fn get_delete_statement(&self) -> Result<Statement, Value> {
    let stmt = "DELETE FROM saved_posts WHERE user_id = $1 AND post_id = $2";

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
    let stmt = "INSERT INTO post_comments (post_id, user_id, comment_id, body, created_at)
      VALUES ($1, $2, $3, $4, $5) RETURNING id";

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
          &Utc::now().naive_utc(),
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

#[derive(Serialize, Deserialize)]
pub struct FetchComments<D, V> {
  sort: Option<Sort>,
  page: Option<i64>,
  #[serde(skip_deserializing)]
  post_id: i32,
  #[serde(skip_deserializing)]
  db_client: D,
  #[serde(skip_deserializing)]
  validated: PhantomData<V>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Sort {
  Latest,
  Oldest,
  Highest,
  Lowest,
}

impl<'a> FetchComments<WithDBClient<'a>, NotValidated> {
  pub fn validate(self) -> Result<FetchComments<WithDBClient<'a>, Validated>, Value> {
    if self.post_id == i32::default() {
      return Err(json!({"message": "Post id not added"}));
    }

    Ok(FetchComments {
      sort: self.sort,
      page: self.page,
      post_id: self.post_id,
      db_client: self.db_client,
      validated: PhantomData,
    })
  }
}

impl FetchComments<NoDBClient, NotValidated> {
  pub fn add_details(
    self,
    db_client: &Client,
    post_id: i32,
  ) -> FetchComments<WithDBClient, NotValidated> {
    FetchComments {
      sort: self.sort,
      page: self.page,
      post_id,
      db_client: WithDBClient(db_client),
      validated: PhantomData,
    }
  }
}

impl<'a, V> FetchComments<WithDBClient<'a>, V> {
  fn get_db_client(&self) -> &'a Client {
    self.db_client.0
  }

  async fn get_fetch_comments_statement(&self) -> Result<Statement, Value> {
    let stmt = "WITH RECURSIVE t(id, body, comment_id, created_at, user_id) AS (
      SELECT id, body, comment_id, created_at, user_id FROM post_comments WHERE post_id = $1
      UNION ALL
      SELECT b.id, b.body, b.comment_id, b.created_at, b.user_id FROM t INNER JOIN post_comments b ON t.comment_id = b.id)
      SELECT t.*, (COUNT(t.id) - 1) replies, u.username author_name, u.id author_id FROM t 
      INNER JOIN users u ON u.id = t.user_id 
      GROUP BY t.id, t.comment_id, t.created_at, t.body, t.user_id, u.id";

    self
      .get_db_client()
      .prepare(stmt)
      .await
      .map_err(|e| json!({"message": e.to_string()}))
  }
}

impl<'a> FetchComments<WithDBClient<'a>, Validated> {
  pub async fn fetch_comments(&self) -> Result<Vec<FetchCommentsResponseParsed>, Value> {
    let res = self
      .get_db_client()
      .query(
        &self.get_fetch_comments_statement().await?,
        &[&self.post_id],
      )
      .await
      .map_err(|e| json!({"message": e.to_string()}))?
      .into_iter()
      .map(|r| FetchCommentsResponse::from_row(&r))
      .collect::<Result<Vec<_>, _>>()?;

    Ok(FetchCommentsResponse::parse(&res, &self.sort))
  }
}

#[derive(Debug, Serialize)]
struct FetchCommentsResponse {
  id: i32,
  body: String,
  comment_id: Option<i32>,
  author: CommentAuthor,
  created_at: NaiveDateTime,
  replies: i64,
}

#[derive(Debug, Serialize)]
pub struct FetchCommentsResponseParsed {
  id: i32,
  body: String,
  author: CommentAuthor,
  created_at: NaiveDateTime,
  reply_count: i64,
  replies: Vec<FetchCommentsResponseParsed>,
}
#[derive(Debug, Clone, Serialize)]
struct CommentAuthor {
  id: i32,
  name: String,
}

impl FetchCommentsResponse {
  fn from_row(r: &Row) -> Result<FetchCommentsResponse, Value> {
    let id = r.try_get::<&str, i32>("id");
    let body = r.try_get::<&str, String>("body");
    let comment_id = r.try_get::<&str, Option<i32>>("comment_id");
    let author_id = r.try_get::<&str, i32>("author_id");
    let author_name = r.try_get::<&str, String>("author_name");
    let replies = r.try_get::<&str, i64>("replies");
    let created_at = r.try_get::<&str, NaiveDateTime>("created_at");

    match (
      id,
      body,
      comment_id,
      author_id,
      author_name,
      replies,
      created_at,
    ) {
      (
        Ok(id),
        Ok(body),
        Ok(comment_id),
        Ok(author_id),
        Ok(author_name),
        Ok(replies),
        Ok(created_at),
      ) => Ok(FetchCommentsResponse {
        id,
        body,
        comment_id,
        replies,
        created_at,
        author: CommentAuthor {
          id: author_id,
          name: author_name,
        },
      }),
      _ => Err(json!({"message": "Error converting postgres to rust type"})),
    }
  }

  fn parse(
    data: &Vec<FetchCommentsResponse>,
    sort: &Option<Sort>,
  ) -> Vec<FetchCommentsResponseParsed> {
    let mut vec = Vec::new();

    FetchCommentsResponse::add_reply(&mut vec, data, None, sort);

    vec
  }

  fn add_reply(
    vec: &mut Vec<FetchCommentsResponseParsed>,
    data: &Vec<FetchCommentsResponse>,
    comment_id: Option<i32>,
    sort: &Option<Sort>,
  ) {
    let res =
      data
        .iter()
        .filter(|d| d.comment_id == comment_id)
        .map(|d| FetchCommentsResponseParsed {
          id: d.id,
          body: d.body.clone(),
          author: d.author.clone(),
          created_at: d.created_at,
          reply_count: d.replies,
          replies: vec![],
        });

    for mut d in res {
      FetchCommentsResponse::add_reply(&mut d.replies, data, Some(d.id), sort);
      vec.push(d);
    }

    match sort {
      Some(s) => match s {
        Sort::Highest => vec.sort_by(|a, b| b.reply_count.cmp(&a.reply_count)),
        Sort::Lowest => vec.sort_by(|a, b| a.reply_count.cmp(&b.reply_count)),
        Sort::Latest => vec.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        Sort::Oldest => vec.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
      },
      None => vec.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
    }
  }
}
