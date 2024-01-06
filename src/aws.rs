use aws_config::{retry::RetryConfig, Region, SdkConfig};

pub async fn get_aws_config() -> SdkConfig {
    aws_config::from_env()
        .region(Region::new("us-east-1"))
        .retry_config(RetryConfig::disabled())
        .load()
        .await
}
