use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use url::Url;

use crate::secretsmanager::get_secret;

#[derive(Debug, Deserialize)]
pub struct DbSecret {
    pub username: String,
    pub password: String,
}

pub async fn get_db_connection_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    let secret: DbSecret = get_secret("rds!db-52c11fca-17e2-4348-a7f4-ba215e56a40b").await;

    let domain = "terraform-20230820160825878800000001.cxp0he9jcakq.us-east-1.rds.amazonaws.com";
    let port = 5432;
    let db_name = "markaronindb";

    let connection_string = format!("postgres://{domain}:{port}/{db_name}");

    let mut url = Url::parse(&connection_string).unwrap();
    url.set_username(&secret.username).unwrap();
    url.set_password(Some(&secret.password)).unwrap();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(url.as_str())
        .await
}
