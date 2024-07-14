use gloo::net::websocket::State as GlooState;

/// The state of the websocket.
/// 
/// Copied from [`State`] to add [`PartialEq`]
///
/// See [`WebSocket.readyState` on MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState)
/// to learn more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// The connection has not yet been established.
    Connecting,
    /// The WebSocket connection is established and communication is possible.
    Open,
    /// The connection is going through the closing handshake, or the close() method has been
    /// invoked.
    Closing,
    /// The connection has been closed or could not be opened.
    Closed,
}

impl From<GlooState> for State {
    fn from(value: GlooState) -> Self {
        use GlooState::*;
        match value {
            Connecting => State::Connecting,
            Open => State::Open,
            Closing => State::Closing,
            Closed => State::Closed,
        }
    }
}
