use cfg_if::cfg_if;
use futures::{select, FutureExt, StreamExt};
use gloo::timers::future::TimeoutFuture;
#[cfg(feature = "state-events")]
use reconnecting_websocket::Event;
use reconnecting_websocket::{Socket, SocketBuilder};

#[path = "./common.rs"]
mod common;

use common::{configure_tracing_once, Input, Output, ECHO_SERVER};
use tracing::{error, info};

#[cfg(all(test, target_arch = "wasm32"))]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[cfg(test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), allow(unused))]
async fn reconnect() {
    use reconnecting_websocket::Error;
    use tracing::trace;

    const SEND_COUNT: usize = 10;

    configure_tracing_once();

    async fn send_messages(socket: &mut Socket<Input, Output>, count: usize) {
        let mut outstanding_packets = Vec::new();

        for i in 0..count {
            outstanding_packets.push(i);
            socket.send(Input::Bar(i)).await.expect("send");
        }

        let mut timeout = TimeoutFuture::new(5000).fuse();

        loop {
            select! {
                r = socket.next() => {
                    let r = r.expect("next None");
                    fn handle_message(o: Result<Output, Error<Input, Output>>, outstanding_packets: &mut Vec<usize>) {
                        match o {
                            Ok(Output::Foo(n)) => {
                                trace!("Output::Foo({n})");
                                outstanding_packets.retain(|v| *v != n);
                            },
                            Err(e) => {
                                error!("next err: {e:?}");
                            }
                        }
                    }
                    cfg_if! {
                        if #[cfg(feature = "state-events")] {
                            match r {
                                Event::Message(m) => handle_message(m, &mut outstanding_packets),
                                Event::State(s) => info!("State changed: {s:?}"),
                            }
                        } else {
                            handle_message(r, &mut outstanding_packets);
                        }
                    }

                    if outstanding_packets.len() == 0 {
                        break;
                    }
                },

                _ = timeout => {
                    panic!("Timed out before receiving all responses (outstanding: {outstanding_packets:?}");
                },
            }
        }
    }

    let mut socket = SocketBuilder::<Input, Output>::new(ECHO_SERVER.to_string()).open().unwrap();

    info!("First test (before reconnect)");
    send_messages(&mut socket, SEND_COUNT).await;

    // Drop the socket
    socket.close_socket(None, Some("test close"));

    info!("Second test (after reconnect)");
    send_messages(&mut socket, SEND_COUNT).await;

    info!("All done");
}
