use std::{
    convert,
    fmt::{self, Debug},
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use cfg_if::cfg_if;
use exponential_backoff::Backoff;
use futures::{
    channel::mpsc::{self, TrySendError, UnboundedReceiver, UnboundedSender},
    ready,
    stream::{self, Fuse, FusedStream},
    Future, SinkExt, Stream, StreamExt,
};
use gloo::{
    net::websocket::{futures::WebSocket, Message, WebSocketError},
    timers::future::TimeoutFuture,
};

use crate::{
    debug, error, event::map_poll, info, trace, Error, Event, SocketInput, SocketOutput, State,
    DEFAULT_BACKOFF_MAX, DEFAULT_BACKOFF_MIN, DEFAULT_MAX_RETRIES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NextPoll {
    Socket,
    Channel,
}

impl Default for NextPoll {
    fn default() -> Self {
        Self::Socket
    }
}

impl NextPoll {
    fn next(self) -> NextPoll {
        use NextPoll::*;
        match self {
            Socket => Channel,
            Channel => Socket,
        }
    }
}

impl IntoIterator for NextPoll {
    type IntoIter = NextPollIter;
    type Item = NextPoll;

    fn into_iter(self) -> Self::IntoIter {
        use NextPoll::*;
        let items = match self {
            Socket => [Socket, Channel],
            Channel => [Channel, Socket],
        };
        NextPollIter { i: 0, items }
    }
}

pub(crate) struct NextPollIter {
    i: usize,
    items: [NextPoll; 2],
}

impl Iterator for NextPollIter {
    type Item = NextPoll;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.items.len() {
            None
        } else {
            self.i += 1;
            Some(self.items[self.i - 1])
        }
    }
}

/// A wrapper around [`WebSocket`] that reconnects when the socket
/// drops. Uses [`Backoff`] to determine the delay between reconnects
///
/// See the [`crate`] documentation for usage and examples
pub struct Socket<I, O> {
    pub(crate) url: String,
    pub(crate) sender: UnboundedSender<I>,
    pub(crate) receiver: UnboundedReceiver<I>,
    pub(crate) socket: Option<WebSocket>,
    pub(crate) state: State,
    pub(crate) backoff: Backoff,
    pub(crate) max_retries: u32,
    pub(crate) retry: u32,
    pub(crate) timeout: Fuse<stream::Once<TimeoutFuture>>,
    pub(crate) next_poll: NextPoll,
    pub(crate) closed: bool,
    pub(crate) _phantom: PhantomData<(I, O)>,
}

impl<I, O> Default for Socket<I, O>
where
    I: SocketInput,
    O: SocketOutput,
    Message: TryFrom<I>,
    <Message as TryFrom<I>>::Error: Debug,
    <O as TryFrom<Message>>::Error: Debug,
{
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded();
        Self {
            url: String::new(),
            sender,
            receiver,
            socket: None,
            state: State::Connecting,
            backoff: Backoff::new(DEFAULT_MAX_RETRIES, DEFAULT_BACKOFF_MIN, DEFAULT_BACKOFF_MAX),
            max_retries: DEFAULT_MAX_RETRIES,
            retry: 0,
            timeout: stream::once(TimeoutFuture::new(0)).fuse(),
            next_poll: NextPoll::Socket,
            closed: false,
            _phantom: PhantomData,
        }
    }
}

impl<I, O> fmt::Debug for Socket<I, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Socket")
            .field("url", &self.url)
            .field("sender", &self.sender)
            .field("receiver", &self.receiver)
            .field("socket.is_some", &self.socket.is_some())
            .field("state", &self.state)
            .field("backoff", &self.backoff)
            .field("max_retries", &self.max_retries)
            .field("retry", &self.retry)
            .field("timeout", &self.timeout)
            .field("next_poll", &self.next_poll)
            .field("closed", &self.closed)
            .finish()
    }
}

impl<I, O> Socket<I, O>
where
    I: SocketInput,
    O: SocketOutput,
    Message: TryFrom<I>,
    <Message as TryFrom<I>>::Error: Debug,
    <O as TryFrom<Message>>::Error: Debug,
{
    /// Send the given `message` for sending
    ///
    /// Internally it is added to a channel which is polled by the [`Stream`] implementation
    /// when the underlying [`WebSocket`] is open and ready to transmit it
    pub async fn send(&mut self, message: I) -> Result<(), TrySendError<I>> {
        self.sender.unbounded_send(message)
    }

    /// Get a sender handle to the message channel
    pub fn get_sender(&self) -> UnboundedSender<I> {
        self.sender.clone()
    }

    /// Close the inner socket with the given `code` and `reason`
    ///
    /// The socket will try and reconnect after a timeout if there are sufficient retries remaining
    ///
    /// This is mainly an implementation detail but it's exposed so it can be used in test code
    /// to force a reconnect. If used in this way it's worth noting that the Closing/Closed state
    /// events won't be emitted
    pub fn close_socket(&mut self, code: Option<u16>, reason: Option<&str>) {
        // Take and drop the socket
        if let Some(socket) = self.socket.take() {
            // Attempt to send the close but don't fail if it can't be sent (the socket could be
            // dead already)
            let _ = socket.close(code, reason);
        }

        // Update our state
        self.state = State::Closed;

        if let Some(timeout) = self.backoff.next(self.retry) {
            debug!("Backoff retry: {}, timeout: {:.3}s", self.retry, timeout.as_secs_f32());
            let millis = timeout.as_millis() as u32;
            self.timeout = stream::once(TimeoutFuture::new(millis)).fuse();
        } else {
            // If we have exceeded our retries the next poll of the stream will close it and error
            // no need to have a timeout in that case
            self.timeout = Self::default().timeout;
        }
    }

    /// Permanently close the reconnecting socket. No further reconnects will be possible
    ///
    /// The socket implements [`FusedStream`] so polling it after close won't panic
    pub fn close(&mut self, code: Option<u16>, reason: Option<&str>) {
        self.closed = true;
        let _ = self.close_socket(code, reason);
    }

    fn map_socket_output(
        output: Option<Result<Message, WebSocketError>>,
    ) -> Option<Result<O, Error<I, O>>> {
        output.map(|result| {
            result
                // Map the gloo socket error
                .map_err(Error::from)
                // Convert the return value into the consumers type
                .map(|message| {
                    debug!("Got output message: {message:?}");
                    O::try_from(message)
                        // Map the consumers try_from error into our error so we can
                        // flatten the result
                        .map_err(Error::<I, O>::from_output)
                })
                // Equivalent to .flatten unstable feature
                .and_then(convert::identity)
        })
    }

    fn map_channel_input(input: Option<I>) -> Option<Result<Message, Error<I, O>>> {
        input.map(|input| {
            debug!("Got input message: {input:?}");
            Message::try_from(input)
                // Map the consumers try_from error into our error
                .map_err(Error::<I, O>::from_input)
        })
    }
}

impl<I, O> FusedStream for Socket<I, O>
where
    I: SocketInput,
    O: SocketOutput,
    Message: TryFrom<I>,
    <Message as TryFrom<I>>::Error: Debug,
    <O as TryFrom<Message>>::Error: Debug,
{
    fn is_terminated(&self) -> bool {
        self.closed
    }
}

impl<I, O> Stream for Socket<I, O>
where
    I: SocketInput,
    O: SocketOutput,
    Message: TryFrom<I>,
    <Message as TryFrom<I>>::Error: Debug,
    <O as TryFrom<Message>>::Error: Debug,
{
    type Item = Event<I, O>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.closed {
            trace!("polled when closed");
            return Poll::Ready(None);
        }

        // Reconnect loop - only loops if socket is none && !self.closed
        while !self.closed {
            // Check we have a socket first
            if let Some(socket) = self.socket.as_ref() {
                // Update our copy of the state and notify if it's changed
                let current_state = socket.state().into();
                if self.state != current_state {
                    self.state = current_state;

                    #[cfg(feature = "state-events")]
                    return Poll::Ready(Some(self.state.into()));
                }
            } else {
                trace!("socket is none");
                ready!(Pin::new(&mut self.timeout).poll_next(cx));

                if self.retry > self.max_retries {
                    error!("retries exceeded. Closing");
                    self.close(None, None);
                    return Poll::Ready(None);
                }

                info!("Reconnecting socket...");
                self.retry += 1;
                match WebSocket::open(&self.url) {
                    Ok(v) => self.socket = Some(v),
                    Err(e) => {
                        error!("WebSocket::open err: {e:?}");
                        return map_poll(Poll::Ready(Some(Err(e.into()))));
                    },
                }

                // Update our state
                self.state = State::Connecting;

                // Announce it if state events are turned on
                #[cfg(feature = "state-events")]
                return Poll::Ready(Some(self.state.into()));
            }

            let next_poll_iter = if self.state == State::Open {
                // If the socket is established we need to poll each future in turn even if we
                // return in between If we return Pending before polling each
                // future, we won't get woken when the unpolled future wakes
                self.next_poll.into_iter()
            } else {
                // If the socket is not established, we want to poll the socket first and if it is
                // still !Open skip polling the incomming message channel since
                // there is nothing we can do with any messages we unqueue from there at this point.
                // The socket has extra waker logic to make sure it wakes up after the socket opens
                // even though it doesn't produce any values at that point
                // so we'll also get woken up and go back to normal polling logic
                NextPoll::Socket.into_iter()
            };

            for next in next_poll_iter {
                // Update so if we return Ready we resume with the right future
                self.next_poll = next.next();

                use NextPoll::*;
                match next {
                    Socket => {
                        // Unwrap ok because we assigned it above if one didn't exist
                        let socket = self.socket.as_mut().unwrap();

                        let poll =
                            Pin::new(&mut socket.next()).poll(cx).map(Self::map_socket_output);
                        match poll {
                            // Just continue to poll the next thing if this is pending
                            Poll::Pending => {},
                            // If it's None (closed) disconnect the socket
                            Poll::Ready(None) => {
                                self.close_socket(None, None);

                                cfg_if! {
                                    if #[cfg(feature = "state-events")] {
                                        // Announce it if state events are turned on
                                        return Poll::Ready(Some(self.state.into()));
                                    } else {
                                        // If not break the next_poll loop to go back to the top of the retry loop
                                        break;
                                    }
                                }
                            },
                            // Covers Ready(Some(Ok | Err))
                            other => return map_poll(other),
                        }
                    },

                    Channel => {
                        // Get the value directly from socket here because plausibly this could be
                        // the 2nd poll of the loop and it could have
                        // updated in between

                        // Unwrap ok because we assigned it above if one didn't exist
                        if State::Open != self.socket.as_mut().unwrap().state().into() {
                            // Don't take anything off the incomming message channel if the socket
                            // isn't open because messages sent to
                            // WebSocket when it's not yet open are lost

                            // Don't poll the channel because the next time we want to be woken is
                            // when the socket is established,
                            // there's no point being woken if the consumer keeps adding data to the
                            // channel
                            trace!("socket not open, skipping channel poll");
                            continue;
                        }
                        if let Poll::Ready(channel_result) = Pin::new(&mut self.receiver.next())
                            .poll(cx)
                            .map(Self::map_channel_input)
                        {
                            if let Some(try_from_result) = channel_result {
                                let payload = match try_from_result {
                                    Err(e) => {
                                        self.next_poll = next.next();
                                        return map_poll(Poll::Ready(Some(Err(e))));
                                    },
                                    Ok(payload) => payload,
                                };

                                // Unwrap ok because we assigned it above if one didn't exist
                                let socket = self.socket.as_mut().unwrap();

                                if let Err(e) = ready!(Pin::new(&mut socket.send(payload))
                                    .poll(cx)
                                    .map_err(Error::<I, O>::from))
                                {
                                    return map_poll(Poll::Ready(Some(Err(e))));
                                }
                            } else {
                                info!("Input channel closed. Closing");
                                self.close(None, None);
                                return Poll::Ready(None);
                            }
                        }
                    },
                }
            }

            if self.socket.is_some() {
                break;
            }
        }

        Poll::Pending
    }
}
