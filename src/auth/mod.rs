pub mod middleware;
pub mod oauth_store;
pub mod routes;
pub mod service_auth;

pub use middleware::{ModeratorAuth, Session};
pub use service_auth::ServiceAuth;

pub const COOKIE_NAME: &str = "debuff_session";
