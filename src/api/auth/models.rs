use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
  password: String,
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

    Ok(CreateAccount {
      first_name: self.first_name.as_ref().unwrap().to_owned(),
      last_name: self.last_name.as_ref().unwrap().to_owned(),
      user_name: self.user_name.as_ref().unwrap().to_owned(),
      password: self.password.as_ref().unwrap().to_owned(),
    })
  }
}

impl CreateAccount {
  pub fn add_to_db(&self) -> Result<(), Value> {
    Ok(())
  }
}
