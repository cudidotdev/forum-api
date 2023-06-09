use deadpool_postgres::Client;
use serde_json::{json, Value};
use tokio_postgres::Statement;

use crate::api::UserAuthDetails;

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
