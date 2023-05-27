use std::{env, vec};

use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_postgres::Client;
use futures_util::TryStreamExt;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
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
  pub async fn validate(&self, db_client: &Client) -> Result<CreateAccount, Value> {
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

    if password.len() < 4 || password.len() > 50 {
      return Err(json!({
        "name": "password",
        "message": "Password should greater than 3 but not more than 50 characters"
      }));
    }

    let is_username_taken = self.is_username_taken(db_client).await;

    if let Err(e) = is_username_taken {
      return Err(json!({
        "name":"user_name",
        "message": e
      }));
    }

    if is_username_taken.unwrap() {
      return Err(json!({
        "name": "user_name",
        "message": "Username is already taken"
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
      password_hash: password_hash.unwrap(),
    })
  }

  async fn is_username_taken(&self, db_client: &Client) -> Result<bool, String> {
    if self.user_name.is_none() {
      return Ok(true);
    }

    let user_name = self.user_name.as_ref().unwrap();

    let stmt = self.get_username_exist_statement(db_client).await;

    if stmt.is_err() {
      return Err("Cannot verify uniqueness of username".to_owned());
    }

    let stmt = stmt.unwrap();

    let res = db_client.query(&stmt, &[user_name]).await;

    if res.is_err() {
      return Err("Cannot verify uniqueness of username".to_owned());
    }

    let res = res.unwrap();
    let row = res.get(0);

    if row.is_none() {
      return Err("Cannot verify uniqueness of username".to_owned());
    }

    row
      .unwrap()
      .try_get("exists")
      .map_err(|_| "Cannot verify uniqueness of username".to_owned())
  }

  async fn get_username_exist_statement(
    &self,
    db_client: &Client,
  ) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "SELECT EXISTS (SELECT 1 FROM users WHERE user_name = $1) as exists";

    db_client.prepare(stmt).await
  }
}

impl CreateAccount {
  pub async fn insert_to_db(&self, db_client: &Client) -> Result<i32, String> {
    let stmt = self.get_insert_statement(db_client).await;

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

    let res = res.unwrap();
    let res = res.get(0).ok_or("No id returned".to_owned());

    if let Err(e) = res {
      return Err(e);
    }

    res.unwrap().try_get("id").map_err(|e| e.to_string())
  }

  async fn get_insert_statement(
    &self,
    db_client: &Client,
  ) -> Result<Statement, tokio_postgres::Error> {
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

#[derive(Serialize, Deserialize)]
pub struct LoginDetails {
  user_name: Option<String>,
  password: Option<String>,
}

pub struct LoginDetailsWithDBClient<'a> {
  user_name: Option<String>,
  password: Option<String>,
  db_client: &'a Client,
}

#[derive(Serialize, Deserialize)]
pub struct UserDetails {
  id: i32,
  user_name: String,
  expires_at: NaiveDateTime,
}

impl LoginDetails {
  pub fn add_db_client(self, db_client: &Client) -> LoginDetailsWithDBClient {
    LoginDetailsWithDBClient {
      user_name: self.user_name,
      password: self.password,
      db_client: db_client,
    }
  }
}

impl<'a> LoginDetailsWithDBClient<'a> {
  pub async fn validate(self) -> Result<UserDetails, Value> {
    if self.user_name.is_none() {
      return Err(json!({
        "name": "user_name",
        "message": "Username is required"
      }));
    }

    if self.password.is_none() {
      return Err(json!({
          "name": "password",
          "message": "Password is required"
      }));
    }

    self
      .get_user_details(self.user_name.as_ref().unwrap())
      .await
  }

  async fn get_user_details(&self, user_name: &String) -> Result<UserDetails, Value> {
    let stmt = self.get_select_statement().await;

    if let Err(e) = stmt {
      return Err(json!({
        "message": format!("Postgres statement error {}", e.to_string())
      }));
    }

    let stmt = stmt.unwrap();

    let res = self.db_client.query(&stmt, &[user_name]).await;

    if let Err(e) = res {
      return Err(json!({
        "message": e.to_string()
      }));
    }

    let vec_row = res.unwrap();
    let row = vec_row.get(0);

    if row.is_none() {
      return Err(json!({
        "name":"user_name",
        "message":"Username does not exists"
      }));
    }

    let row = row.unwrap();

    let id = row.try_get::<&str, i32>("id");
    let user_name = row.try_get::<&str, String>("user_name");
    let password_hash = row.try_get::<&str, &str>("password_hash");

    if !(id.is_ok() && user_name.is_ok() && password_hash.is_ok()) {
      return Err(json!({
        "message": "Error converting from postgres to rust"
      }));
    }

    let res = bcrypt::verify(self.password.as_ref().unwrap(), password_hash.unwrap());

    if res.is_err() {
      return Err(json!({
        "message": "Error verifying password"
      }));
    }

    if !res.unwrap() {
      return Err(json!({
        "name": "password",
        "message": "Wrong password"
      }));
    }

    Ok(UserDetails {
      id: id.unwrap(),
      user_name: user_name.unwrap(),
      expires_at: Utc::now().naive_utc() + Duration::weeks(2),
    })
  }

  async fn get_select_statement(&self) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "SELECT id, user_name, password_hash FROM users WHERE user_name = $1";

    self.db_client.prepare(stmt).await
  }
}

impl UserDetails {
  pub fn get_jwt(&self) -> String {
    let jwt_secret = env::var("JWT_SECRET").unwrap_or("ItsPublic".to_owned());

    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();

    self.sign_with_key(&key).unwrap()
  }
}
