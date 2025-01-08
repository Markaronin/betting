use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use url::Url;

pub async fn get_db_connection_pool(
    db_username: &str,
    db_password: &str,
) -> Result<Pool<Postgres>, sqlx::Error> {
    let url_encoded_password: String =
        url::form_urlencoded::byte_serialize(db_password.as_bytes()).collect();

    let domain = "terraform-20230820160825878800000001.cxp0he9jcakq.us-east-1.rds.amazonaws.com";
    let port = 5432;
    let db_name = "markaronindb";

    let connection_string = format!("postgres://{domain}:{port}/{db_name}");

    let mut url = Url::parse(&connection_string).unwrap();
    url.set_username(db_username).unwrap();
    url.set_password(Some(&url_encoded_password)).unwrap();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(url.as_str())
        .await
}
