use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;

use crate::{dynamodb::get_user, jwt::create_jwt, user_id_cookie::ExtractUserId, AppState};

pub async fn login_page(
    possible_user_id_cookie: Option<ExtractUserId>,
    State(app_state): State<AppState>,
) -> Response {
    match possible_user_id_cookie {
        Some(_) => Redirect::to("/").into_response(),
        None => Html(
            app_state
                .engine
                .render("login", &tera::Context::new())
                .unwrap(),
        )
        .into_response(),
    }
}

#[derive(Deserialize)]
pub struct LoginForm {
    user_id: String,
}
pub async fn login(
    jar: CookieJar,
    State(app_state): State<AppState>,
    Form(request): Form<LoginForm>,
) -> Response {
    match get_user(&app_state.dynamodb_client, &request.user_id).await {
        Some(_) => {
            let jwt = create_jwt(&request.user_id, &app_state.secret);

            let mut cookie = Cookie::new("betting-auth", jwt);

            cookie.make_permanent();

            (jar.add(cookie), Redirect::to("/")).into_response()
        }
        None => Redirect::to("/login").into_response(),
    }
}
