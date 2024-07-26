#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! trace {
    ($($_:tt)*) => {};
}

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! debug {
    ($($_:tt)*) => {};
}

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! info {
    ($($_:tt)*) => {};
}

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! warn {
    ($($_:tt)*) => {};
}

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! error {
    ($($_:tt)*) => {};
}
