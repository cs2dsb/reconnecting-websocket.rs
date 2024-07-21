use std::{fmt::Debug, marker::PhantomData, time::Duration};

use exponential_backoff::Backoff;
use gloo::net::websocket::{futures::WebSocket, Message};

use crate::{
    constants::DEFAULT_STABLE_CONNECTION_TIMEOUT, info, Error, Socket, SocketInput, SocketOutput,
    DEFAULT_BACKOFF_MAX, DEFAULT_BACKOFF_MIN, DEFAULT_MAX_RETRIES,
};

/// Builder for [`Socket`]
/// Uses the DEFAULT_* consts for backoff and retry config
#[derive(Debug)]
pub struct SocketBuilder<I, O> {
    url: String,
    backoff_min: Duration,
    backoff_max: Option<Duration>,
    max_retries: u32,
    stable_timeout: Duration,
    _phantom: PhantomData<(I, O)>,
}

impl<I, O> SocketBuilder<I, O>
where
    I: SocketInput,
    O: SocketOutput,
    Message: TryFrom<I>,
    <Message as TryFrom<I>>::Error: Debug,
    <O as TryFrom<Message>>::Error: Debug,
{
    /// Create a new builder from the given url with other config set to defaults
    pub fn new(url: String) -> Self {
        Self {
            url,
            backoff_min: DEFAULT_BACKOFF_MIN,
            backoff_max: DEFAULT_BACKOFF_MAX,
            max_retries: DEFAULT_MAX_RETRIES,
            stable_timeout: DEFAULT_STABLE_CONNECTION_TIMEOUT,
            _phantom: PhantomData,
        }
    }

    /// Update the builder url
    pub fn set_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }

    /// Update the minimum backoff duration (must be > 0 millis)
    pub fn set_backoff_min(mut self, backoff_min: Duration) -> Self {
        self.backoff_min = backoff_min;
        self
    }

    /// Update the maximum backoff duration (if set must be < u32::MAX millis)
    pub fn set_backoff_max(mut self, backoff_max: Option<Duration>) -> Self {
        self.backoff_max = backoff_max;
        self
    }

    /// Update the maximum number of retry attempts
    pub fn set_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Update the stable timeout. Must be <= u32::MAX millis
    ///
    /// This determines how long a connection needs to stay open after a retry before it
    /// is considered stable and the retry counter is reset to 0
    pub fn set_stable_timeout(mut self, stable_timeout: Duration) -> Self {
        self.stable_timeout = stable_timeout;
        self
    }

    /// Attempts to create a reconnecting websocket and do the initial open
    /// It's set up to error at this poing because the kind of errors that can occur here are likely
    /// fatal (See [`gloo::net::websocket::futures::WebSocket::open`] for details). These could
    /// be panics but the consumer may want to display the error to the user or fallback to
    /// plain http
    pub fn open(self) -> Result<Socket<I, O>, Error<I, O>> {
        let SocketBuilder { url, backoff_min, backoff_max, max_retries, stable_timeout, .. } = self;

        if backoff_min == Duration::ZERO {
            return Err(Error::InvalidConfig("backoff_min must be > 0".to_string()));
        }

        if let Some(max) = backoff_max.as_ref() {
            if max.as_millis() > (u32::MAX as u128) {
                return Err(Error::InvalidConfig(
                    "backoff_max must be <= u32::MAX millis".to_string(),
                ));
            }
        }

        if max_retries == 0 {
            return Err(Error::InvalidConfig("backoff_retries must be > 0".to_string()));
        }

        if stable_timeout.as_millis() > (u32::MAX as u128) {
            return Err(Error::InvalidConfig(
                "stable_timeout must be <= u32::MAX millis".to_string(),
            ));
        }
        let stable_timeout_millis = stable_timeout.as_millis() as u32;

        info!("Opening reconnecting websocket to {url}");
        let socket = WebSocket::open(&url)?;

        let backoff = Backoff::new(max_retries, backoff_min, backoff_max);

        Ok(Socket {
            url,
            socket: Some(socket),
            backoff,
            max_retries,
            stable_timeout_millis,
            ..Default::default()
        })
    }
}
