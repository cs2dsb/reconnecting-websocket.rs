use std::{fmt::Debug, marker::PhantomData, time::Duration};

use exponential_backoff::Backoff;
use gloo::net::websocket::{futures::WebSocket, Message};

use crate::{
    info, Error, Socket, SocketInput, SocketOutput, DEFAULT_BACKOFF_MAX, DEFAULT_BACKOFF_MIN,
    DEFAULT_MAX_RETRIES,
};

#[derive(Debug)]
pub struct SocketBuilder<I, O> {
    url: String,
    backoff_min: Duration,
    backoff_max: Option<Duration>,
    max_retries: u32,
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
    pub fn new(url: String) -> Self {
        Self {
            url,
            backoff_min: DEFAULT_BACKOFF_MIN,
            backoff_max: DEFAULT_BACKOFF_MAX,
            max_retries: DEFAULT_MAX_RETRIES,
            _phantom: PhantomData,
        }
    }

    pub fn set_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }

    pub fn set_backoff_min(mut self, backoff_min: Duration) -> Self {
        self.backoff_min = backoff_min;
        self
    }

    pub fn set_backoff_max(mut self, backoff_max: Option<Duration>) -> Self {
        self.backoff_max = backoff_max;
        self
    }

    pub fn set_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Attempts to create a reconnecting websocket and do the initial open
    /// It's set up this way because the kind of errors that can occur here are likely fatal (See
    /// [`gloo::net::websocket::futures::WebSocket::open`] for details) These could be panics but
    /// the consumer may want to display the error to the user or fallback to plain http
    pub fn open(self) -> Result<Socket<I, O>, Error<I, O>> {
        let SocketBuilder { url, backoff_min, backoff_max, max_retries, .. } = self;

        if backoff_min == Duration::ZERO {
            return Err(Error::InvalidConfig("backoff_min must be > 0".to_string()));
        }

        if let Some(max) = backoff_max.as_ref() {
            if max.as_millis() > (u32::MAX as u128) {
                return Err(Error::InvalidConfig("backoff_max must be <= u32::MAX".to_string()));
            }
        }

        if max_retries == 0 {
            return Err(Error::InvalidConfig("backoff_retries must be > 0".to_string()));
        }

        info!("Opening reconnecting websocket to {url}");
        let socket = WebSocket::open(&url)?;

        let backoff = Backoff::new(max_retries, backoff_min, backoff_max);

        Ok(Socket { url, socket: Some(socket), backoff, max_retries, ..Default::default() })
    }
}
