use std::fmt::Debug;

use gloo::{
    net::websocket::{Message, WebSocketError},
    utils::errors::JsError,
};

use crate::{error, SocketInput, SocketOutput};

/// Errors returned by [`crate::Socket`] and [`crate::SocketBuilder`]
#[derive(Debug, thiserror::Error)]
pub enum Error<I, O>
where
    I: SocketInput,
    O: SocketOutput,
    Message: TryFrom<I>,
    <Message as TryFrom<I>>::Error: Debug,
    <O as TryFrom<Message>>::Error: Debug,
{
    /// Errors from the underlying [`gloo::net::websocket::futures::WebSocket`]
    ///
    /// These are unlikely to be fatal, they mostly express various ways the websocket has
    /// disconnected or encountered an error that requires reconnecting
    #[error("WebSocketError: {0}")]
    WebSocketError(WebSocketError),

    /// Javascript errors returned by either [`crate::get_proto_and_host`] or the underlying
    /// [`gloo::net::websocket::futures::WebSocket`]
    ///
    /// These errors are often fatal errors like the URL provided is invalid, the port is
    /// firewalled or other miscellaneous errors from the browser. Look at
    /// [`gloo::net::websocket::futures::WebSocket::open`] to work out what the cases are and
    /// how to handle them
    #[error("JsError: {0}")]
    JsError(#[from] JsError),

    /// Invalid configuration provided to [`crate::SocketBuilder`]
    ///
    /// These errors are only returned from the bulder and are all fatal
    #[error("InvalidConfig: {0}")]
    InvalidConfig(String),

    /// Input errors returned by the consumers implementation of <[`crate::Message`] as
    /// [`TryFrom<I>`]>
    ///
    /// If these errors are fatal is dependent on consumers implementation of [`TryFrom<I>`]
    #[error("Input TryFrom(Message) Err: {0:?}")]
    InputError(<Message as TryFrom<I>>::Error),

    /// Output errors returned by the consumers implementation of <O as [`TryFrom<Message>`]>
    ///
    /// If these errors are fatal is dependent on consumers implementation of [`TryFrom<Message>`]
    #[error("Output TryFrom<Message> Err: {0:?}")]
    OutputError(<O as TryFrom<Message>>::Error),
}

impl<I, O> From<WebSocketError> for Error<I, O>
where
    I: SocketInput,
    O: SocketOutput,
    Message: TryFrom<I>,
    <Message as TryFrom<I>>::Error: Debug,
    <O as TryFrom<Message>>::Error: Debug,
{
    fn from(err: WebSocketError) -> Self {
        error!("WebSocketError: {err:?}");
        Self::WebSocketError(err)
    }
}

impl<I, O> Error<I, O>
where
    I: SocketInput,
    O: SocketOutput,
    Message: TryFrom<I>,
    <Message as TryFrom<I>>::Error: Debug,
    <O as TryFrom<Message>>::Error: Debug,
{
    pub(crate) fn from_input(err: <I as TryInto<Message>>::Error) -> Self {
        error!("Input TryInto<Message> Error: {err:?}");
        Self::InputError(err)
    }

    pub(crate) fn from_output(err: <O as TryFrom<Message>>::Error) -> Self {
        error!("Output TryFrom<Message> Error: {err:?}");
        Self::OutputError(err)
    }
}
