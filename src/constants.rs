use std::time::Duration;

/// The minimum delay for exponential backoff
/// See [`exponential_backoff::Backoff`] for details
/// Must be > 0 millis
pub const DEFAULT_BACKOFF_MIN: Duration = Duration::from_millis(100);

/// The maximum delay for exponential backoff
/// See [`exponential_backoff::Backoff`] for details
/// Must be <= u32::MAX millis
pub const DEFAULT_BACKOFF_MAX: Option<Duration> = Some(Duration::from_secs(60));

/// The maximum number of retries. The stream will close after this is exceeded
pub const DEFAULT_MAX_RETRIES: u32 = u32::MAX;

/// How long to wait before considering a retried connection stable again (and setting retries back
/// to 0) Must be <= u32::MAX millis
pub const DEFAULT_STABLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(90);
