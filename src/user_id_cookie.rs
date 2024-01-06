use axum::{async_trait, extract::FromRequestParts, http::request::Parts, response::Redirect};
use axum_extra::extract::CookieJar;

use crate::{jwt::validate_and_extract_user_id, AppState};

pub struct ExtractUserId(pub String);

#[async_trait]
impl FromRequestParts<AppState> for ExtractUserId {
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();

        match jar.get("betting-auth") {
            Some(auth_cookie) => {
                match validate_and_extract_user_id(auth_cookie.value(), &state.secret) {
                    Ok(user_id) => Ok(ExtractUserId(user_id)),
                    Err(e) => {
                        println!("{e:#?}");
                        Err(Redirect::to("/login"))
                    }
                }
            }
            None => Err(Redirect::to("/login")),
        }
    }
}
