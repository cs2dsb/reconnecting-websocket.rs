#[doc(hidden)]
#[macro_export]
macro_rules! trace {
    ($($_:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! debug {
    ($($_:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! info {
    ($($_:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! warn {
    ($($_:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! error {
    ($($_:tt)*) => {};
}
