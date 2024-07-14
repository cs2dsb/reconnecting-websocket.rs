[![Crate](https://img.shields.io/crates/v/reconnecting-websocket.svg)](https://crates.io/crates/reconnecting-websocket)
[![Documentation](https://docs.rs/reconnecting-websocket/badge.svg)](https://docs.rs/reconnecting-websocket)
[![Workflow Status](https://github.com/cs2dsb/reconnecting-websocket.rs/actions/workflows/build.yml/badge.svg)](https://github.com/cs2dsb/reconnecting-websocket.rs/actions?query=workflow%3A%22build%22)
![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# reconnecting-websocket

A wrapper around [`WebSocket`] that reconnects when the socket
drops. Uses [`Backoff`] to determine the delay between reconnects

## Features

* `tracing` - enables the [`tracing`] crate and logs everything it's doing
* `state-events` - changes the Item type of the stream to be an enum that is either a message or
  a status change Both are enabled by default

## Usage

Input means stuff you want to send from this client to the server

Outut means stuff you want to receive back from the server

1. Implement [`TryFrom`] for [`Message`] for your input type
    * The [`TryFrom::Error`] type must implement [`Debug`]
1. Implement [`TryFrom<Message>`] for your output type
    * The [`TryFrom::Error`] type must implement [`Debug`]
1. Both input and output need to implement [`Unpin`] and, if using tracing feature, [`Debug`]
1. Use [`SocketBuilder`] to set the URL and configure backoff. [`get_proto_and_host`] can help
   constructing the URL relative to the current `window.location`
1. Call [`SocketBuilder::open`] to connect the socket. The errors `open` returns are likely fatal
   (invalid URL, blocked port), see [`WebSocket::open`] for details. The first connect is done
   in the builder so it fails fast if these fatal errors occur but the same kind of error can
   also occur on any reconnect and be returned by the [`Socket`] [`Stream`] implementation
1. The returned [`Socket`] can then be polled to get incoming messages. [`Socket::send`] can be
   called to send messages or [`Socket::get_sender`] can be used to get an [`UnboundedSender`].
   [`Socket::close`] or dropping it will drop the inner [`WebSocket`] which sends a close frame
   and cleans up the event handlers

## Example

[`tests/reconnect.rs`](tests/reconnect.rs)

## Docs

[Code docs](https://cs2dsb.github.io/reconnecting-websocket.rs)

## License

[MIT](LICENSE)

[`Backoff`]: https://docs.rs/exponential_backoff/latest/exponential_backoff/struct.Backoff.html
[`WebSocket`]: https://docs.rs/gloo-net/latest/gloo_net/websocket/futures/struct.WebSocket.html
[`WebSocket::open`]:https://docs.rs/gloo-net/latest/gloo_net/websocket/futures/struct.WebSocket.html#method.open
[`Stream`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
[`UnboundedSender`]: https://docs.rs/futures/latest/futures/channel/mpsc/struct.UnboundedSender.html
[`tracing`]: https://docs.rs/tracing/latest/index.html
[`TryFrom`]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html
[`Message`]: https://docs.rs/gloo-net/latest/gloo_net/websocket/enum.Message.html
[`TryFrom::Error`]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html#associatedtype.Error
[`Debug`]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
[`Unpin`]: https://doc.rust-lang.org/std/marker/trait.Unpin.html
[`Socket`]: https://docs.rs/reconnecting-websocket/latest/reconnecting_websocket/struct.Socket.html
[`Socket::send`]: https://docs.rs/reconnecting-websocket/latest/reconnecting_websocket/struct.Socket.html#method.send
[`Socket::get_sender`]: https://docs.rs/reconnecting-websocket/latest/reconnecting_websocket/struct.Socket.html#method.get_sender
[`Socket::close`]: https://docs.rs/reconnecting-websocket/latest/reconnecting_websocket/struct.Socket.html#method.close
[`get_proto_and_host`]: https://docs.rs/reconnecting-websocket/latest/fn.reconnecting_websocket/.html
[`SocketBuilder`]: https://docs.rs/reconnecting-websocket/latest/reconnecting_websocket/struct.SocketBuilder.html
[`SocketBuilder::open`]: https://docs.rs/reconnecting-websocket/latest/reconnecting_websocket/struct.SocketBuilder.html#method.open
[`TryFrom<Message>`]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html