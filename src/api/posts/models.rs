use std::marker::PhantomData;

use crate::api::UserAuthDetails;
use actix_web::http::StatusCode;
use chrono::{NaiveDateTime, Utc};
use deadpool_postgres::Client;
use futures_util::{future, TryStreamExt};
use lazy_static::lazy_static;
use postgres_types::{FromSql, ToSql};
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_postgres::{Row, Statement};

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

#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "color")]
enum Color {
  #[postgres(name = "green")]
  Green,
  #[postgres(name = "blue")]
  Blue,
  #[postgres(name = "red")]
  Red,
  #[postgres(name = "yellow")]
  Yellow,
  #[postgres(name = "purple")]
  Purple,
  #[postgres(name = "violet")]
  Violet,
}

impl<D, U, V> CreatePostDetails<D, U, V> {
  fn get_random_color(&self) -> Color {
    match rand::thread_rng().gen_range(0..6) {
      0 => Color::Green,
      1 => Color::Blue,
      2 => Color::Red,
      3 => Color::Yellow,
      4 => Color::Purple,
      5 => Color::Violet,
      _ => Color::Red,
    }
  }
}

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

    if self.title.len() == 0 {
      return Err((
        StatusCode::BAD_REQUEST,
        json!({"name": "title", "message": "Post title has no content"}),
      ));
    }

    if self.body.len() == 0 {
      return Err((
        StatusCode::BAD_REQUEST,
        json!({"name": "body", "message": "Post body has no content"}),
      ));
    }

    if self.title.len() > 100 {
      return Err((
        StatusCode::BAD_REQUEST,
        json!({"name": "title", "message": "Post title should not have more 100 characters"}),
      ));
    }

    if self.body.len() > 1000 {
      return Err((
        StatusCode::BAD_REQUEST,
        json!({"name": "body", "message": "Post body should not have more 1000 characters"}),
      ));
    }

    lazy_static! {
      static ref RE: Result<Regex, regex::Error> = Regex::new(r"[^A-Za-z\s]+");
    }

    let mut all_under_51 = true;

    self.topics = self
      .topics
      .iter()
      .map(|s| {
        let mut s = String::from(s.trim());

        s.make_ascii_lowercase();
        s.get_mut(0..1).map(|a| a.make_ascii_uppercase());

        if RE.is_ok() {
          s = String::from(RE.as_ref().unwrap().replace_all(s.as_str(), "").to_string());
        }

        all_under_51 = all_under_51 && s.len() <= 50;

        s
      })
      .filter(|s| s.len() > 0)
      .collect();

    if !all_under_51 {
      return Err((
        StatusCode::BAD_REQUEST,
        json!({"message": "Topic names should not be more than 50 characters"}),
      ));
    }

    if self.topics.len() == 0 {
      return Err((
        StatusCode::BAD_REQUEST,
        json!({"name": "topics", "message": "Please add topics"}),
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

impl<'a, D, V> CreatePostDetails<D, UserDetails<'a>, V> {
  fn get_user_details(&self) -> &UserAuthDetails {
    self.user_details.0
  }
}

impl<'a, U> CreatePostDetails<DBClient<'a>, U, Validated> {
  fn get_db_client(&self) -> &'a Client {
    self.db_client.0
  }

  async fn get_create_post_statment(&self) -> Result<Statement, (StatusCode, Value)> {
    let stmt =
      "INSERT INTO posts(title, body, user_id, created_at) VALUES ($1, $2, $3, $4) RETURNING id";
    self.get_db_client().prepare(stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }

  async fn get_insert_topics_statement(&self) -> Result<Statement, (StatusCode, Value)> {
    let mut stmt = "INSERT INTO topics (name, color, created_at) VALUES ".to_owned();

    let mut i = 0;
    let n = self.topics.len() * 3;

    while i < n {
      stmt.push_str(&format!("(${}, ${}, ${})", i + 1, i + 2, i + 3));
      if i + 3 != n {
        stmt.push_str(",")
      }
      i = i + 3;
    }

    stmt += "ON CONFLICT (name) DO NOTHING";

    self.get_db_client().prepare(&stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }

  fn get_insert_topics_params(&self) -> Vec<Box<dyn ToSql + Sync>> {
    let mut v: Vec<Box<dyn ToSql + Sync>> = Vec::new();

    self.topics.iter().for_each(|t| {
      v.push(Box::new(t.to_owned()));
      v.push(Box::new(self.get_random_color()));
      v.push(Box::new(Utc::now().naive_utc()));
    });

    v
  }

  async fn get_insert_post_and_topics_ids_statement(
    &self,
  ) -> Result<Statement, (StatusCode, Value)> {
    let mut stmt = "INSERT INTO posts_topics_relationship (post_id, topic_id) (SELECT $1, id FROM topics WHERE name IN (".to_owned();

    let mut i = 1;
    let n = self.topics.len() + 1;

    while i < n {
      i += 1;
      stmt += &format!("${}", i);
      if i != n {
        stmt += ",";
      }
    }

    stmt += "))";

    self.get_db_client().prepare(&stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }

  fn get_insert_post_and_topics_ids_params(&self, post_id: &i32) -> Vec<Box<dyn ToSql + Sync>> {
    let mut v: Vec<Box<dyn ToSql + Sync>> = vec![Box::new(*post_id)];

    self.topics.iter().for_each(|t_id| {
      v.push(Box::new(String::from(t_id)));
    });

    v
  }
}

impl<'a> CreatePostDetails<DBClient<'a>, UserDetails<'a>, Validated> {
  pub async fn create_post(&self) -> Result<i32, (StatusCode, Value)> {
    let res = future::join(self.insert_post(), self.insert_topics()).await;

    let post_id = res.0?;

    self.insert_post_and_topics_ids(post_id).await?;

    Ok(post_id)
  }

  async fn insert_post(&self) -> Result<i32, (StatusCode, Value)> {
    self
      .get_db_client()
      .query(
        &self.get_create_post_statment().await?,
        &[
          &self.title,
          &self.body,
          &self.get_user_details().id,
          &Utc::now().naive_utc(),
        ],
      )
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({ "message": e.to_string() }),
        )
      })?
      .get(0)
      .ok_or((
        StatusCode::NOT_FOUND,
        json!({ "message": "No id returned" }),
      ))?
      .try_get("id")
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({ "message": e.to_string() }),
        )
      })
  }

  async fn insert_topics(&self) -> Result<(), (StatusCode, Value)> {
    self
      .get_db_client()
      .query_raw(
        &self.get_insert_topics_statement().await?,
        self.get_insert_topics_params(),
      )
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })?
      .try_collect::<Vec<Row>>()
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })
      .map(|_| ())
  }

  async fn insert_post_and_topics_ids(&self, post_id: i32) -> Result<(), (StatusCode, Value)> {
    self
      .get_db_client()
      .query_raw(
        &self.get_insert_post_and_topics_ids_statement().await?,
        self.get_insert_post_and_topics_ids_params(&post_id),
      )
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })
      .map(|_| ())
  }
}

#[derive(Serialize, Deserialize)]
pub struct FetchPosts<D, U, V> {
  sort: Option<String>,
  limit: Option<String>,
  page: Option<String>,
  topics: Option<Vec<String>>,
  #[serde(skip_deserializing)]
  db_client: D,
  #[serde(skip_deserializing)]
  user_details: U,
  #[serde(skip_deserializing)]
  validated: PhantomData<V>,
}

#[derive(Debug, Serialize)]
struct FetchPostsResponse {
  title: String,
  body: String,
  topics: Vec<String>,
  author: PostAuthor,
  created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
struct PostAuthor {
  id: i32,
  name: String,
}

impl<D, U> FetchPosts<D, U, NotValidated> {
  pub fn validate(self) -> Result<FetchPosts<D, U, Validated>, (StatusCode, Value)> {
    Ok(FetchPosts {
      sort: self.sort,
      limit: self.limit,
      page: self.page,
      topics: self.topics,
      db_client: self.db_client,
      user_details: self.user_details,
      validated: PhantomData,
    })
  }
}

impl<U, V> FetchPosts<NoDBClient, U, V> {
  pub fn add_db_client(self, db_client: &Client) -> FetchPosts<DBClient, U, V> {
    FetchPosts {
      sort: self.sort,
      limit: self.limit,
      page: self.page,
      topics: self.topics,
      db_client: DBClient(db_client),
      user_details: self.user_details,
      validated: PhantomData,
    }
  }
}

impl<D, V> FetchPosts<D, NoUserDetails, V> {
  pub fn add_user_details(self, user_details: &UserAuthDetails) -> FetchPosts<D, UserDetails, V> {
    FetchPosts {
      sort: self.sort,
      limit: self.limit,
      page: self.page,
      topics: self.topics,
      db_client: self.db_client,
      user_details: UserDetails(user_details),
      validated: PhantomData,
    }
  }
}

impl<'a, U, V> FetchPosts<DBClient<'a>, U, V> {
  pub fn get_db_client(&self) -> &'a Client {
    self.db_client.0
  }
}

impl<'a> FetchPosts<DBClient<'a>, NoUserDetails, Validated> {
  pub async fn get_select_statement(&self) -> Result<Statement, (StatusCode, Value)> {
    let stmt = "SELECT p.title, p.body, u.id author_id, u.username author_name, p.created_at, ARRAY_AGG(t.name) topics FROM posts p 
     INNER JOIN posts_topics_relationship r ON p.id = r.post_id 
     INNER JOIN topics t ON t.id = r.topic_id
     INNER JOIN users u ON u.id = p.user_id GROUP BY p.id, u.username, u.id";

    self.get_db_client().prepare(stmt).await.map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "message": e.to_string() }),
      )
    })
  }

  pub async fn fetch_posts(&self) -> Result<Value, (StatusCode, Value)> {
    let res = self
      .get_db_client()
      .query(&self.get_select_statement().await?, &[])
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({
            "message": e.to_string()
          }),
        )
      })?
      .into_iter()
      .map(|r| FetchPostsResponse::from_row(&r))
      .collect::<Result<Vec<_>, _>>()?;

    serde_json::to_value(res).map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message": e.to_string()}),
      )
    })
  }
}

impl<'a> FetchPosts<DBClient<'a>, UserDetails<'a>, Validated> {
  pub async fn fetch_posts(&self) -> Result<Value, (StatusCode, Value)> {
    todo!();
  }
}

impl FetchPostsResponse {
  fn from_row(row: &Row) -> Result<FetchPostsResponse, (StatusCode, Value)> {
    let title = row.try_get::<&str, String>("title");
    let body = row.try_get::<&str, String>("body");
    let topics = row.try_get::<&str, Vec<String>>("topics");
    let author_name = row.try_get::<&str, String>("author_name");
    let author_id = row.try_get::<&str, i32>("author_id");
    let created_at = row.try_get::<&str, NaiveDateTime>("created_at");

    match (title, body, topics, author_id, author_name, created_at) {
      (Ok(title), Ok(body), Ok(topics), Ok(author_id), Ok(author_name), Ok(created_at)) => {
        Ok(FetchPostsResponse {
          title,
          body,
          topics,
          author: PostAuthor {
            id: author_id,
            name: author_name,
          },
          created_at,
        })
      }
      _ => Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({"message":"Error converting postgres types" }),
      )),
    }
  }
}
