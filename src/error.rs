use std::fmt::Debug;

use gloo::{
    net::websocket::{Message, WebSocketError},
    utils::errors::JsError,
};

use crate::{error, SocketInput, SocketOutput};

/// Errors returned by this crate
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
    #[error("WebSocketError: {0}")]
    WebSocketError(WebSocketError),

    /// Javascript errors returned by either [`crate::get_proto_and_host`] or the underlying
    /// [`gloo::net::websocket::futures::WebSocket`]
    #[error("JsError: {0}")]
    JsError(#[from] JsError),

    /// Invalid configuration provided to [`crate::SocketBuilder`]
    #[error("InvalidConfig: {0}")]
    InvalidConfig(String),

    /// Input errors returned by the consumers implementation of <[`crate::Message`] as
    /// [`TryFrom<I>`]>
    #[error("Input TryFrom(Message) Err: {0:?}")]
    InputError(<Message as TryFrom<I>>::Error),

    /// Output errors returned by the consumers implementation of <O as [`TryFrom<Message>`]>
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
