use aws_config::{retry::RetryConfig, BehaviorVersion, Region};
use envconfig::Envconfig;
use serde::{de::DeserializeOwned, Deserialize};

use crate::axum_lambda_util::is_running_on_lambda;

async fn get_secret<T>(client: &aws_sdk_secretsmanager::Client, name: &str) -> T
where
    T: DeserializeOwned,
{
    let req = client
        .get_secret_value()
        .set_secret_id(Some(name.to_string()));
    let resp = req.send().await.unwrap().secret_string.unwrap();
    serde_json::from_str(&resp).unwrap()
}

#[derive(Envconfig)]
pub struct Secrets {
    pub auth_secret: String,
    pub db_username: String,
    pub db_password: String,
}
impl Secrets {
    async fn from_lambda_secretsmanager() -> Self {
        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .retry_config(RetryConfig::disabled())
            .load()
            .await;

        let client = aws_sdk_secretsmanager::Client::new(&aws_config);

        #[derive(Deserialize)]
        pub struct AuthSecret {
            #[serde(rename = "auth-token-signer")]
            pub auth_token_signer: String,
        }
        let auth_secret = get_secret::<AuthSecret>(&client, "markaronin-auth")
            .await
            .auth_token_signer;

        #[derive(Deserialize)]
        pub struct DbSecret {
            pub username: String,
            pub password: String,
        }
        let db_secret: DbSecret = get_secret::<DbSecret>(&client, "betting-db-user").await;

        Self {
            auth_secret,
            db_username: db_secret.username,
            db_password: db_secret.password,
        }
    }

    pub async fn load() -> Self {
        if is_running_on_lambda() {
            Self::from_lambda_secretsmanager().await
        } else {
            dotenvy::dotenv().unwrap();
            Self::init_from_env().unwrap()
        }
    }
}
