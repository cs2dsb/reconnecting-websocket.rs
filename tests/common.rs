use std::{num::ParseIntError, sync::Once};

use reconnecting_websocket::Message;
use time::format_description::well_known::Iso8601;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{performance_layer, MakeWebConsoleWriter};

pub const ECHO_SERVER: &str = "wss://echo.websocket.org/";

pub fn configure_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        // .without_time()   // std::time is not available in browsers
        .with_timer(UtcTime::new(Iso8601::DEFAULT))
        .with_writer(MakeWebConsoleWriter::new()); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    // Install these as subscribers to tracing events
    tracing_subscriber::registry().with(fmt_layer).with(perf_layer).init();
}

/// Configures tracing inside a Once block so multiple calls don't panic
pub fn configure_tracing_once() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| configure_tracing());
}

#[derive(Debug)]
pub enum Input {
    Bar(usize),
}

impl TryFrom<Input> for Message {
    type Error = ();

    fn try_from(value: Input) -> Result<Self, Self::Error> {
        let Input::Bar(i) = value;
        Ok(Message::Text(format!("Bar({i})")))
    }
}

#[derive(Debug)]
pub enum Output {
    #[allow(unused)]
    Foo(usize),
}

impl TryFrom<Message> for Output {
    type Error = ParseIntError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        if let Message::Text(text) = value {
            let num = text.replace("Bar(", "").replace(")", "");
            let n: usize = num.parse()?;
            Ok(Self::Foo(n))
        } else {
            panic!("Only Message::Text supported in this test. Got {value:?}");
        }
    }
}
