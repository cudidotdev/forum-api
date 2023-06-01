use std::{
  env,
  future::{ready, Ready},
};

use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};
use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_postgres::Client;
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use tokio_postgres::Statement;

#[derive(Serialize, Deserialize)]
pub struct CreateAccountDetails {
  username: Option<String>,
  password: Option<String>,
  confirm_password: Option<String>,
}

#[derive(Debug)]
pub struct CreateAccountDetailsWithDBClient<'a> {
  username: Option<String>,
  password: Option<String>,
  confirm_password: Option<String>,
  db_client: &'a Client,
}

#[derive(Serialize, Deserialize)]
pub struct LoginDetails {
  username: Option<String>,
  password: Option<String>,
}

pub struct LoginDetailsWithDBClient<'a> {
  username: Option<String>,
  password: Option<String>,
  db_client: &'a Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthDetails {
  pub id: i32,
  pub username: String,
  pub expires_at: NaiveDateTime,
}

pub struct UserAuth {
  pub details: Option<UserAuthDetails>,
}

impl CreateAccountDetails {
  pub fn add_db_client(self, db_client: &Client) -> CreateAccountDetailsWithDBClient {
    CreateAccountDetailsWithDBClient {
      username: self.username,
      password: self.password,
      confirm_password: self.confirm_password,
      db_client: db_client,
    }
  }
}

impl<'a> CreateAccountDetailsWithDBClient<'a> {
  pub async fn insert_to_db(&self) -> Result<UserAuthDetails, Value> {
    let stmt = self
      .get_insert_statement()
      .await
      .map_err(|e| json!({ "message": format!("Postgres statement error {}", e.to_string()) }))?;

    let (username, _) = self.validate_details().await?;

    self
      .db_client
      .query(
        &stmt,
        &[
          &self.username,
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
      .map(|id| UserAuthDetails {
        id,
        username,
        expires_at: Utc::now().naive_utc() + Duration::weeks(2),
      })
  }

  async fn get_insert_statement(&self) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "INSERT INTO users (username, password_hash, created_at)
                      VALUES ($1, $2, $3)
                      RETURNING id";

    self.db_client.prepare(stmt).await
  }

  async fn validate_details(&self) -> Result<(String, String), Value> {
    if self.username.is_none() {
      return Err(json!({
          "name": "username",
          "message": "Username is required"
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

    let username = self.username.as_ref().unwrap();
    let password = self.password.as_ref().unwrap();

    if username.len() > 50 {
      return Err(json!({
        "name": "username",
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
        "name":"username",
        "message": e
      })
    })?;

    if is_username_taken {
      return Err(json!({
        "name": "username",
        "message": "Username is already taken"
      }));
    };

    Ok((username.to_owned(), password.to_owned()))
  }

  async fn is_username_taken(&self) -> Result<bool, String> {
    let username = self
      .username
      .as_ref()
      .ok_or("Username is required".to_owned())?;

    let stmt = self
      .get_username_exist_statement()
      .await
      .map_err(|_| "Cannot verify uniqueness of username".to_owned())?;

    self
      .db_client
      .query(&stmt, &[username])
      .await
      .map_err(|_| "Cannot verify uniqueness of username".to_owned())?
      .get(0)
      .ok_or("Cannot verify uniqueness of username".to_owned())?
      .try_get("exists")
      .map_err(|_| "Cannot verify uniqueness of username".to_owned())
  }

  async fn get_username_exist_statement(&self) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "SELECT EXISTS (SELECT 1 FROM users WHERE username = $1) as exists";

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

impl LoginDetails {
  pub fn add_db_client(self, db_client: &Client) -> LoginDetailsWithDBClient {
    LoginDetailsWithDBClient {
      username: self.username,
      password: self.password,
      db_client,
    }
  }
}

impl<'a> LoginDetailsWithDBClient<'a> {
  pub async fn validate(self) -> Result<UserAuthDetails, Value> {
    if self.username.is_none() {
      return Err(json!({
        "name": "username",
        "message": "Username is required"
      }));
    }

    if self.password.is_none() {
      return Err(json!({
          "name": "password",
          "message": "Password is required"
      }));
    }

    self.get_user_details(self.username.as_ref().unwrap()).await
  }

  async fn get_user_details(&self, username: &String) -> Result<UserAuthDetails, Value> {
    let stmt = self
      .get_select_statement()
      .await
      .map_err(|e| json!({ "message": format!("Postgres statement error {}", e.to_string()) }))?;

    let vec_row = self
      .db_client
      .query(&stmt, &[username])
      .await
      .map_err(|e| {
        json!({
          "message": e.to_string()
        })
      })?;

    let row = vec_row.get(0).ok_or(json!({
      "name":"username",
      "message":"Username does not exists"
    }))?;

    let id = row.try_get::<&str, i32>("id");
    let username = row.try_get::<&str, String>("username");
    let password_hash = row.try_get::<&str, &str>("password_hash");

    if !(id.is_ok() && username.is_ok() && password_hash.is_ok()) {
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
      username: username.unwrap(),
      expires_at: Utc::now().naive_utc() + Duration::weeks(2),
    })
  }

  async fn get_select_statement(&self) -> Result<Statement, tokio_postgres::Error> {
    let stmt = "SELECT id, username, password_hash FROM users WHERE username = $1";

    self.db_client.prepare(stmt).await
  }
}

impl UserAuthDetails {
  pub fn from_jwt(token: &str) -> Result<UserAuthDetails, String> {
    let jwt_secret = env::var("JWT_SECRET").unwrap_or("ItsPublic".to_owned());

    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();

    token
      .verify_with_key(&key)
      .map_err(|e| e.to_string())
      .and_then(|u: UserAuthDetails| {
        if Utc::now().naive_utc() >= u.expires_at {
          Err("Token expired".to_owned())
        } else {
          Ok(u)
        }
      })
  }

  pub fn to_jwt(self) -> String {
    let jwt_secret = env::var("JWT_SECRET").unwrap_or("ItsPublic".to_owned());

    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();

    self.sign_with_key(&key).unwrap_or(String::new())
  }
}

impl FromRequest for UserAuth {
  type Error = Error;
  type Future = Ready<Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
    ready(Ok(UserAuth {
      details: req.extensions().get::<UserAuthDetails>().cloned(),
    }))
  }
}
