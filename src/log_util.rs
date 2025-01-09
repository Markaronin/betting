use env_logger::builder;
use log::LevelFilter;

pub fn init_default_debug_logger() {
    builder()
        .filter_level(LevelFilter::Debug)
        .filter_module("aws_config", LevelFilter::Warn)
        .filter_module("aws_credential_types", LevelFilter::Warn)
        .filter_module("aws_smithy_client", LevelFilter::Info)
        .filter_module("aws_smithy_runtime_api", LevelFilter::Info)
        .filter_module("aws_smithy_runtime", LevelFilter::Info)
        .filter_module("aws_smithy_http_tower", LevelFilter::Info)
        .filter_module("hyper", LevelFilter::Info)
        .filter_module("rustls", LevelFilter::Info)
        .filter_module("tracing", LevelFilter::Warn)
        .init();
}
