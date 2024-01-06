use aws_sdk_secretsmanager::Client;
use serde::de::DeserializeOwned;

use crate::aws::get_aws_config;

pub async fn get_secret<T: DeserializeOwned>(name: &str) -> T {
    let shared_config = get_aws_config().await;
    let client = Client::new(&shared_config);

    let req = client
        .get_secret_value()
        .set_secret_id(Some(name.to_string()));
    let resp = req.send().await.unwrap().secret_string.unwrap();
    serde_json::from_str(&resp).unwrap()
}
