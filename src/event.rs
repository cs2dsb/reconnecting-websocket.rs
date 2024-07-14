use std::{fmt::Debug, task::Poll};

use cfg_if::cfg_if;
use gloo::net::websocket::Message;

use crate::{Error, SocketInput, SocketOutput, State};

cfg_if! {
    if #[cfg(feature = "state-events")] {
        pub enum Event<I, O>
        where
            I: SocketInput,
            O: SocketOutput,
            Message: TryFrom<I>,
            <Message as TryFrom<I>>::Error: Debug,
            <O as TryFrom<Message>>::Error: Debug,
        {
            Message(Result<O, Error<I, O>>),
            State(State),
        }

        impl<I, O> From<Result<O, Error<I, O>>> for Event<I, O>
        where
            I: SocketInput,
            O: SocketOutput,
            Message: TryFrom<I>,
            <Message as TryFrom<I>>::Error: Debug,
            <O as TryFrom<Message>>::Error: Debug,
        {
            fn from(value: Result<O, Error<I, O>>) -> Self {
                Self::Message(value)
            }
        }

        impl<I, O> From<State> for Event<I, O>
        where
            I: SocketInput,
            O: SocketOutput,
            Message: TryFrom<I>,
            <Message as TryFrom<I>>::Error: Debug,
            <O as TryFrom<Message>>::Error: Debug,
        {
            fn from(value: State) -> Self {
                Self::State(value)
            }
        }

        pub(crate) fn map_poll<I, O>(
            poll: Poll<Option<Result<O, Error<I, O>>>>,
        ) -> Poll<Option<Event<I, O>>>
        where
            I: SocketInput,
            O: SocketOutput,
            Message: TryFrom<I>,
            <Message as TryFrom<I>>::Error: Debug,
            <O as TryFrom<Message>>::Error: Debug,
        {
            poll.map(|option| option.map(|result| Event::<_, _>::from(result)))
        }
} else {
        pub type Event<I, O> = Result<O, Error<I, O>>;

        // Does nothing
        pub(crate) fn map_poll<I, O>(
            poll: Poll<Option<Event<I, O>>>,
        ) -> Poll<Option<Event<I, O>>>
        where
            I: SocketInput,
            O: SocketOutput,
            Message: TryFrom<I>,
            <Message as TryFrom<I>>::Error: Debug,
            <O as TryFrom<Message>>::Error: Debug,
        {
            poll
        }
    }
}
