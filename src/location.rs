use std::fmt::Debug;

use gloo::utils::errors::JsError;
use web_sys::{
    js_sys::{Error as JsSysError, ReferenceError},
    window,
};

/// The protocol extracted from `window.location`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpProtocol {
    /// Insecure http
    Http,
    /// Secure https
    Https,
}

impl ToString for HttpProtocol {
    fn to_string(&self) -> String {
        use HttpProtocol::*;
        match self {
            Http => "http".to_string(),
            Https => "https".to_string(),
        }
    }
}

impl Into<WebSocketProtocol> for HttpProtocol {
    fn into(self) -> WebSocketProtocol {
        use HttpProtocol::*;
        match self {
            Http => WebSocketProtocol::Ws,
            Https => WebSocketProtocol::Wss,
        }
    }
}

/// A websocket protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketProtocol {
    /// Insecure websocket
    Ws,
    /// Secure websocket
    Wss,
}

impl ToString for WebSocketProtocol {
    fn to_string(&self) -> String {
        use WebSocketProtocol::*;
        match self {
            Ws => "ws".to_string(),
            Wss => "wss".to_string(),
        }
    }
}

/// Helper function to get the protocol and host from `window.location`
pub fn get_proto_and_host() -> Result<(HttpProtocol, String), JsError> {
    let location =
        window().ok_or(JsSysError::from(ReferenceError::new("window global was none")))?.location();

    let host = location.host().map_err(|v| {
        JsSysError::try_from(v).unwrap_or(JsSysError::new("failed to get location.host"))
    })?;

    let proto = location.protocol().map_err(|v| {
        JsSysError::try_from(v).unwrap_or(JsSysError::new("faild to get location.protocol"))
    })?;

    let proto = match proto.as_ref() {
        "http:" => Ok(HttpProtocol::Http),
        "https:" => Ok(HttpProtocol::Https),
        other => Err(JsSysError::new(&format!(
            "Unexpected protocol \"{other}\" (Expected http: or https:)"
        ))),
    }?;

    Ok((proto, host))
}
