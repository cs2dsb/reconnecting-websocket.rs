use std::fmt::Debug;

use gloo::{
    net::websocket::{Message, WebSocketError},
    utils::errors::JsError,
};

use crate::{error, SocketInput, SocketOutput};

#[derive(Debug, thiserror::Error)]
pub enum Error<I, O>
where
    I: SocketInput,
    O: SocketOutput,
    Message: TryFrom<I>,
    <Message as TryFrom<I>>::Error: Debug,
    <O as TryFrom<Message>>::Error: Debug,
{
    #[error("WebSocketError: {0}")]
    WebSocketError(WebSocketError),

    #[error("JsError: {0}")]
    JsError(#[from] JsError),

    #[error("InvalidConfig: {0}")]
    InvalidConfig(String),

    #[error("Input TryInto<Message> Err: {0:?}")]
    InputError(<Message as TryFrom<I>>::Error),

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
