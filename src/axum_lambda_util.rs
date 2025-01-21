use axum::Router;
use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    request, HeaderValue, Method,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn is_running_on_lambda() -> bool {
    std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
}

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

pub async fn run_router(router: Router) {
    let router = router.layer(get_default_cors_policy());

    if is_running_on_lambda() {
        // To run with AWS Lambda runtime, wrap in our `LambdaLayer`
        let app = tower::ServiceBuilder::new()
            .layer(axum_aws_lambda::LambdaLayer::default())
            .service(router);

        lambda_http::run(app).await.unwrap();
    } else {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router).await.unwrap();
    }
}
