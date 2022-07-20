use actix_web::rt::task::JoinHandle;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// Compose multiple layers into a `tracingd's` suscriber
///
/// ## Implementation Notes
/// We return `impl Subscriber` to avoid having to spell out the actual type
/// of the returned type which can be complx
/// We also need to call out that the returned trait object also implements
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
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

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    actix_web::rt::task::spawn_blocking(move || current_span.in_scope(f))
}
