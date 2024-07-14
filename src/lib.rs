//! A wrapper around [`WebSocket`] that reconnects when the socket
//! drops. Uses [`Backoff`] to determine the delay between reconnects
//!
//! # Features
//!
//! * `tracing` - enables the [`tracing`] crate and logs everything it's doing
//! * `state-events` - changes the Item type of the stream to be an enum that is either a message or
//!   a
//! status change Both are enabled by default
//!
//! # Usage
//!
//! Input means stuff you want to send from this client to the server
//!
//! Outut means stuff you want to receive back from the server
//!
//! 1. Implement [`TryFrom`] for [`Message`] for your input type
//!     * The [`TryFrom::Error`] type must implement [`Debug`]
//! 1. Implement [`TryFrom<Message>`] for your output type
//!     * The [`TryFrom::Error`] type must implement [`Debug`]
//! 1. Both input and output need to implement [`Unpin`] and, if using tracing feature, [`Debug`]
//! 1. Use [`SocketBuilder`] to set the URL and configure backoff. [`get_proto_and_host`] can help
//!    constructing the URL relative to the current `window.location`
//! 1. Call [`SocketBuilder::open`] to connect the socket. The errors `open` returns are likely
//!    fatal (invalid URL, blocked port), see [`WebSocket::open`] for details. The first connect is
//!    done in the builder so it fails fast if these fatal errors occur but the same kind of error
//!    can also occur on any reconnect and be returned by the [`Socket`] [`Stream`] implementation
//! 1. The returned [`Socket`] can then be polled to get incoming messages. [`Socket::send`] can be
//!    called to send messages or [`Socket::get_sender`] can be used to get an [`UnboundedSender`].
//!    [`Socket::close`] or dropping it will drop the inner [`WebSocket`] which sends a close frame
//!    and cleans up the event handlers
//!
//! # Example
//!
//! `tests/reconnect.rs`
//! ```rust
#![doc = include_str!("../tests/reconnect.rs")]
//! ```
//! 
//! [`WebSocket`]: gloo::net::websocket::futures::WebSocket
//! [`Backoff`]: exponential_backoff::Backoff
//! [`WebSocket::open`]: gloo::net::websocket::futures::WebSocket::open
//! [`Stream`]: futures::Stream
//! [`UnboundedSender`]: futures::channel::mpsc::UnboundedSender

// TODO: Replace unbounded with a reasonable bounded channel

#![warn(missing_docs)]

use std::fmt::Debug;

use cfg_if::cfg_if;
#[doc(inline)]
/// Re-export of [`gloo::net::websocket::Message`].
pub use gloo::net::websocket::Message;

mod error;
pub use error::Error;

mod location;
pub use location::{get_proto_and_host, HttpProtocol, WebSocketProtocol};

mod event;
pub use event::Event;

mod constants;
pub use constants::{DEFAULT_BACKOFF_MAX, DEFAULT_BACKOFF_MIN, DEFAULT_MAX_RETRIES};

mod builder;
pub use builder::SocketBuilder;

mod state;
pub use state::State;

mod socket;
pub use socket::Socket;

// Plumbing for making it work with and without tracing
cfg_if! {
    if #[cfg(feature = "tracing")] {
        #[allow(unused_imports)]
        use tracing::{trace, debug, info, warn, error};

        /// Trait expressing the requirements for a socket input type
        /// You don't need to implement it directly, there is a blanked implementation for types that implement
        /// [`Unpin`], [`Debug`], <[`Message`] as [`TryFrom<Self>`]>
        pub trait SocketInput: Unpin + Debug + Sized
        where
            Message: TryFrom<Self>,
            <Message as TryFrom<Self>>::Error: Debug
        {}

        /// Trait expressing the requirements for a socket output type
        /// You don't need to implement it directly, there is a blanked implementation for types that implement
        /// [`Unpin`], [`Debug`], <`Self` as [`TryFrom<Message>`]>
        pub trait SocketOutput: Unpin + TryFrom<Message> + Debug
        where <Self as TryFrom<Message>>::Error: Debug {}

        impl<T: Unpin + Debug + Sized> SocketInput for T
        where
            Message: TryFrom<T>,
            <Message as TryFrom<T>>::Error: Debug
        {}

        impl<T: Unpin + TryFrom<Message> + Debug> SocketOutput for T
        where <T as TryFrom<Message>>::Error: Debug {}
    } else {
        mod dummy_tracing;

        pub trait SocketInput: Unpin + Sized
        where
            Message: TryFrom<Self>,
            <Message as TryFrom<Self>>::Error: Debug
        {}

        pub trait SocketOutput: Unpin + TryFrom<Message>
        where <Self as TryFrom<Message>>::Error: Debug {}

        impl<T: Unpin + Sized> SocketInput for T
        where
            Message: TryFrom<T>,
            <Message as TryFrom<T>>::Error: Debug
        {}

        impl<T: Unpin + TryFrom<Message>> SocketOutput for T
        where <T as TryFrom<Message>>::Error: Debug {}
    }
}
