use rocket::http::Status;
use rocket::Request;
use rocket::request::{FromRequest, Outcome};
use rocket_session_store::{Session, SessionResult};
use crate::models::user::UserModel;
use crate::{LoginSessionData, SessionData};

pub struct AuthUser {
    pub session: LoginSessionData
}

#[derive(Debug)]
pub enum UserError {
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = UserError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let sess = match request.guard::<Session<SessionData>>().await {
            Outcome::Success(sess ) => sess,
            _ => return Outcome::Forward(Status::Unauthorized)
        };
        let sess = match sess.get().await {
            Ok(Some(sess)) => {
                sess
            }
            _ => return Outcome::Forward(Status::Unauthorized),
        };
        if let Some(login) = &sess.login {
            Outcome::Success(Self { session: login.clone() })
        } else {
            Outcome::Forward(Status::Unauthorized)
        }
    }
}