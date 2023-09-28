mod auth;
mod hashtags;
mod posts;
mod users;

pub use auth::models::UserAuth;
pub use auth::models::UserAuthDetails;
pub use auth::view as auth;
pub use hashtags::view as hashtags;
pub use posts::view as post;
pub use users::view as user;

pub mod handler_utils {
  use super::UserAuthDetails;
  use deadpool_postgres::Client;
  #[derive(Default, Debug)]
  pub struct NoDBClient;
  pub struct WithDBClient<'a>(pub &'a Client);
  #[derive(Default, Debug)]
  pub struct NoUserDetails;
  pub struct WithUserDetails<'a>(pub &'a UserAuthDetails);
  #[derive(Default, Debug)]
  pub struct NotValidated;
  pub struct Validated;
}
