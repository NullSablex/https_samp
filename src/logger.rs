//! Thin wrapper around `samp::log` that prepends the plugin prefix
//! (`[https_samp]`) to every line. Use via the `info!`, `warn!`, and `error!`
//! macros re-exported at the crate root.

pub const PREFIX: &str = "[https_samp]";

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        samp::log::info!("{} {}", $crate::logger::PREFIX, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        samp::log::warn!("{} {}", $crate::logger::PREFIX, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        samp::log::error!("{} {}", $crate::logger::PREFIX, format_args!($($arg)*))
    };
}
