use deadpool_postgres::Client;
use futures_util::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio_postgres::{Row, Statement};

#[derive(Serialize, Deserialize)]
pub struct CreateAccountReq {
  first_name: Option<String>,
  last_name: Option<String>,
  user_name: Option<String>,
  password: Option<String>,
  confirm_password: Option<String>,
}

#[derive(Debug)]
pub struct CreateAccount {
  first_name: String,
  last_name: String,
  user_name: String,
  password_hash: String,
}

impl CreateAccountReq {
  pub fn validate(&self) -> Result<CreateAccount, Value> {
    if self.first_name.is_none() {
      return Err(json!({
        "name": "first_name",
        "message": "First name is required"
      }));
    }

    if self.last_name.is_none() {
      return Err(json!({
          "name": "last_name",
          "message": "Last name is required"
      }));
    }

    if self.user_name.is_none() {
      return Err(json!({
          "name": "user_name",
          "message": "User name is required"
      }));
    }

    if self.password.is_none() {
      return Err(json!({
          "name": "password",
          "message": "Password is required"
      }));
    }

    if self.password != self.confirm_password {
      return Err(json!({
          "name": "confirm_password",
          "message": "Passwords does not match"
      }));
    }

    let first_name = self.first_name.as_ref().unwrap();
    let last_name = self.last_name.as_ref().unwrap();
    let user_name = self.user_name.as_ref().unwrap();
    let password = self.password.as_ref().unwrap();

    if first_name.len() > 50 {
      return Err(json!({
        "name": "first_name",
        "message": "Names should not be more than 50 characters"
      }));
    }

    if last_name.len() > 50 {
      return Err(json!({
        "name": "last-name",
        "message": "Names should not be more than 50 characters"
      }));
    }

    if user_name.len() > 50 {
      return Err(json!({
        "name": "user_name",
        "message": "Names should not be more than 50 characters"
      }));
    }

    if password.len() > 50 {
      return Err(json!({
        "name": "password",
        "message": "Password should not be more than 50 characters"
      }));
    }

    let password_hash = bcrypt::hash(password, 6);

    if let Err(e) = password_hash {
      return Err(json!({
        "name": "password",
        "message": format!("Error hashing password\n{}", e.to_string())

      }));
    }

    Ok(CreateAccount {
      first_name: first_name.to_owned(),
      last_name: last_name.to_owned(),
      user_name: user_name.to_owned(),
      password_hash: password.to_owned(),
    })
  }
}

impl CreateAccount {
  pub async fn insert_to_db(&self, db_client: &Client) -> Result<(), String> {
    let stmt = CreateAccount::get_insert_statement(db_client).await;

    if let Err(e) = stmt {
      return Err(format!("Postgres statement error {}", e.to_string()));
    }

    let stmt = stmt.unwrap();

    let exec_res = db_client
      .query_raw(&stmt, self.get_insert_parameters())
      .await;

    if let Err(e) = exec_res {
      return Err(format!("e {}", e.to_string()));
    }

    let res: Result<Vec<Row>, tokio_postgres::Error> = exec_res.unwrap().try_collect().await;

    if let Err(e) = res {
      return Err(format!("e {}", e.to_string()));
    }

    for r in res.unwrap() {
      println!("{:#?}", r.columns()[0].name());
    }

    // println!("{res:#?}");

    Ok(())
  }

  async fn get_insert_statement(db_client: &Client) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "INSERT INTO users (first_name, last_name, user_name, password_hash)
                      VALUES ($1, $2, $3, $4)
                      RETURNING id";

    db_client.prepare(stmt).await
  }

  fn get_insert_parameters(&self) -> Vec<&String> {
    vec![
      &self.first_name,
      &self.last_name,
      &self.user_name,
      &self.password_hash,
    ]
  }
}
