use log::LevelFilter;
use simple_logger::SimpleLogger;

pub fn get_default_logger() -> SimpleLogger {
    SimpleLogger::new()
        .with_module_level("actix_web", LevelFilter::Debug)
        .with_module_level("aws_sdk_s3", LevelFilter::Info)
        .with_module_level("aws_sdk_dynamodb", LevelFilter::Info)
        .with_module_level("aws_smithy_client", LevelFilter::Info)
        .with_module_level("aws_smithy_runtime_api", LevelFilter::Info)
        .with_module_level("aws_smithy_runtime", LevelFilter::Info)
        .with_module_level("aws_smithy_http_tower", LevelFilter::Info)
        .with_module_level("hyper", LevelFilter::Info)
        .with_module_level("reqwest", LevelFilter::Info)
        .with_module_level("rustls", LevelFilter::Info)
        .with_module_level("aws_config", LevelFilter::Warn)
        .with_module_level("aws_credential_types", LevelFilter::Warn)
        .with_module_level("tracing", LevelFilter::Warn)
}

pub fn init_default_debug_logger() {
    get_default_logger()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap()
}
