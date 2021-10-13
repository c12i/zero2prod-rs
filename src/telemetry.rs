use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};


/// Compose multiple layers into a `tracingd's` suscriber
///
/// ## Implementation Notes
/// We return `impl Subscriber` to avoid having to spell out the actual type
/// of the returned type which can be complx
/// We also need to call out that the returned trait object also implements
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
	// fall back to printing all logs at inflo level or above
	// if RUST_LOG env var has not been set
	let env_filter =
			EnvFilter::try_from_default_env().unwrap_or_else(|_| -> EnvFilter {EnvFilter::new(env_filter)});
	let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
	// the `with` method is provided by `SubsciberExt`, an extension trait for `Subscriber` exposed by `tracing_subscriber`
	Registry::default()
			.with(env_filter)
			.with(JsonStorageLayer)
			.with(formatting_layer)
}

/// Register a subscriber as a global default to process span data
///
/// Should only be called once
pub fn initialize_subscriber(subscriber: impl Subscriber + Send + Sync) {
	// redirect all Actix web `log` events to subscriber
	LogTracer::init().expect("Error setting logger");
	// `set_global_default` can be used by applications to specify what subscriber should be used to process spans
	set_global_default(subscriber).expect("Error getting subscriber");
}