use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

// Notice the impl keyword. Understand this usage better.
pub fn init_subscriber(subscriber: impl tracing::Subscriber + Send + Sync) {
    // First capture all the logs to the tracing.
    LogTracer::init().expect("Failed to set logger.");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn create_subscriber(
    name: String,
    env_filter: String,
) -> impl tracing::Subscriber + Send + Sync {
    // Check for RUST_LOG env variable and if not set use env_filter
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(BunyanFormattingLayer::new(name, std::io::stdout))
}
