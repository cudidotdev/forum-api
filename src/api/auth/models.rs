use std::env;

use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_postgres::Client;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use tokio_postgres::Statement;

#[derive(Serialize, Deserialize)]
pub struct CreateAccountDetails {
  first_name: Option<String>,
  last_name: Option<String>,
  user_name: Option<String>,
  password: Option<String>,
  confirm_password: Option<String>,
}

#[derive(Debug)]
pub struct CreateAccountDetailsWithDBClient<'a> {
  first_name: Option<String>,
  last_name: Option<String>,
  user_name: Option<String>,
  password: Option<String>,
  confirm_password: Option<String>,
  db_client: &'a Client,
}

impl CreateAccountDetails {
  pub fn add_db_client(self, db_client: &Client) -> CreateAccountDetailsWithDBClient {
    CreateAccountDetailsWithDBClient {
      first_name: self.first_name,
      last_name: self.last_name,
      user_name: self.user_name,
      password: self.password,
      confirm_password: self.confirm_password,
      db_client: db_client,
    }
  }
}

impl<'a> CreateAccountDetailsWithDBClient<'a> {
  pub async fn insert_to_db(&self) -> Result<i32, Value> {
    let stmt = self
      .get_insert_statement()
      .await
      .map_err(|e| json!({ "message": format!("Postgres statement error {}", e.to_string()) }))?;

    self.validate_details().await?;

    self
      .db_client
      .query(
        &stmt,
        &[
          &self.first_name,
          &self.last_name,
          &self.user_name,
          &self.hash_password()?,
          &Utc::now().naive_utc(),
        ],
      )
      .await
      .map_err(|e| json!({ "message": format!("e {}", e.to_string()) }))?
      .get(0)
      .ok_or("No id returned".to_owned())
      .map_err(|e| json!({ "message": e }))?
      .try_get("id")
      .map_err(|e| json!({"message": e.to_string()}))
  }

  async fn get_insert_statement(&self) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "INSERT INTO users (first_name, last_name, user_name, password_hash, created_at)
                      VALUES ($1, $2, $3, $4, $5)
                      RETURNING id";

    self.db_client.prepare(stmt).await
  }

  async fn validate_details(&self) -> Result<(), Value> {
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

    let is_username_taken = self.is_username_taken().await.map_err(|e| {
      json!({
        "name":"user_name",
        "message": e
      })
    })?;

    if is_username_taken {
      return Err(json!({
        "name": "user_name",
        "message": "Username is already taken"
      }));
    };

    Ok(())
  }

  async fn is_username_taken(&self) -> Result<bool, String> {
    let user_name = self
      .user_name
      .as_ref()
      .ok_or("Username is required".to_owned())?;

    let stmt = self
      .get_username_exist_statement()
      .await
      .map_err(|_| "Cannot verify uniqueness of username".to_owned())?;

    self
      .db_client
      .query(&stmt, &[user_name])
      .await
      .map_err(|_| "Cannot verify uniqueness of username".to_owned())?
      .get(0)
      .ok_or("Cannot verify uniqueness of username".to_owned())?
      .try_get("exists")
      .map_err(|_| "Cannot verify uniqueness of username".to_owned())
  }

  async fn get_username_exist_statement(&self) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "SELECT EXISTS (SELECT 1 FROM users WHERE user_name = $1) as exists";

    self.db_client.prepare(stmt).await
  }

  fn hash_password(&self) -> Result<String, Value> {
    if self.password.is_none() {
      return Err(json!({
          "name": "password",
          "message": "Password is required"
      }));
    }

    bcrypt::hash(self.password.as_ref().unwrap(), 6).map_err(|e| {
      json!({
        "name": "password",
        "message": format!("Error hashing password\n{}", e.to_string())

      })
    })
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
pub struct UserAuthDetails {
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
  pub async fn validate(self) -> Result<UserAuthDetails, Value> {
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

  async fn get_user_details(&self, user_name: &String) -> Result<UserAuthDetails, Value> {
    let stmt = self
      .get_select_statement()
      .await
      .map_err(|e| json!({ "message": format!("Postgres statement error {}", e.to_string()) }))?;

    let vec_row = self
      .db_client
      .query(&stmt, &[user_name])
      .await
      .map_err(|e| {
        json!({
          "message": e.to_string()
        })
      })?;

    let row = vec_row.get(0).ok_or(json!({
      "name":"user_name",
      "message":"Username does not exists"
    }))?;

    let id = row.try_get::<&str, i32>("id");
    let user_name = row.try_get::<&str, String>("user_name");
    let password_hash = row.try_get::<&str, &str>("password_hash");

    if !(id.is_ok() && user_name.is_ok() && password_hash.is_ok()) {
      return Err(json!({
        "message": "Error converting from postgres to rust"
      }));
    }

    let wrong_password = !bcrypt::verify(self.password.as_ref().unwrap(), password_hash.unwrap())
      .map_err(|_| {
      json!({
        "message": "Error verifying password"
      })
    })?;

    if wrong_password {
      return Err(json!({
        "name": "password",
        "message": "Wrong password"
      }));
    }

    Ok(UserAuthDetails {
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

impl UserAuthDetails {
  pub fn to_jwt(self) -> String {
    let jwt_secret = env::var("JWT_SECRET").unwrap_or("ItsPublic".to_owned());

    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();

    self.sign_with_key(&key).unwrap()
  }
}
