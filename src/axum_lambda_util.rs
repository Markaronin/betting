use std::net::SocketAddr;

use axum::Router;
use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    request, HeaderValue, Method,
};
use lambda_web::{is_running_on_lambda, run_hyper_on_lambda, LambdaError};
use tower_http::cors::{AllowOrigin, CorsLayer};

fn get_default_cors_policy() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(
            |origin: &HeaderValue, _request_parts: &request::Parts| {
                (is_running_on_lambda() && origin.as_bytes().ends_with(b"betting.markaronin.com"))
                    || (!is_running_on_lambda()
                        && origin.as_bytes().starts_with(b"http://localhost:"))
            },
        ))
        .allow_methods([
            Method::CONNECT,
            Method::DELETE,
            Method::GET,
            Method::HEAD,
            Method::OPTIONS,
            Method::PATCH,
            Method::POST,
            Method::PUT,
            Method::TRACE,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true)
}

pub async fn run_router(router: Router) -> Result<(), LambdaError> {
    let router = router.layer(get_default_cors_policy());

    if is_running_on_lambda() {
        run_hyper_on_lambda(router).await?;
    } else {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .await?;
    }
    Ok(())
}
