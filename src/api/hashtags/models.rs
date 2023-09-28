use actix_web::http::StatusCode;
use deadpool_postgres::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_postgres::{Row, Statement};

use crate::api::handler_utils::{NoDBClient, WithDBClient};

#[derive(Deserialize)]
pub struct FetchTrendingHashtags<D> {
  #[serde(skip_deserializing)]
  db_client: D,
}

impl<'a> FetchTrendingHashtags<NoDBClient> {
  pub fn add_db_client(self, db_client: &'a Client) -> FetchTrendingHashtags<WithDBClient<'a>> {
    FetchTrendingHashtags {
      db_client: WithDBClient(db_client),
    }
  }
}

impl<'a> FetchTrendingHashtags<WithDBClient<'a>> {
  pub async fn fetch(&self) -> Result<Value, (StatusCode, Value)> {
    self
      .get_db_client()
      .query(&self.get_select_statement().await?, &[])
      .await
      .map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"message": e.to_string()}),
        )
      })?
      .into_iter()
      .map(|row| TrendingHashtagsResponse::value(row))
      .collect::<Result<_, _>>()
      .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
  }

  async fn get_select_statement(&self) -> Result<Statement, (StatusCode, Value)> {
    let stmt = r#"
      SELECT COUNT(hashtag_id) score, name, color::TEXT, created_at 
      FROM posts_hashtags_relationship ph LEFT JOIN hashtags h ON ph.hashtag_id = h.id
      WHERE now() - created_at < interval '48 hours' GROUP BY h.id ORDER BY score DESC LIMIT 7"#;

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

struct TrendingHashtagsResponse;

impl TrendingHashtagsResponse {
  pub fn value(row: Row) -> Result<Value, Value> {
    let name = row
      .try_get::<&str, String>("name")
      .map_err(|_| json!({"message": "Error converting hashtag name postgres to rust type"}))?;
    let color = row
      .try_get::<&str, String>("color")
      .map_err(|_| json!({"message": "Error converting color postgres to rust type"}))?;

    Ok(json!({
      "name": name,
      "color": color
    }))
  }
}
