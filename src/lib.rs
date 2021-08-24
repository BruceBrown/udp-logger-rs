#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]
#![allow(dead_code)]

//! This crate provides logging macros that incorporates key/value logging
//! and a logger that sends all logs to a UDP port.
//!
//! # Examples
//! ```
//! use std::vec::Vec;
//! use udp_logger_rs::info;
//!
//! let ctx: Vec<(String, String)> = vec![
//!   ("cat_1".into(), "chashu".into()),
//!   ("cat_2".into(), "nori".into()),
//! ];
//!
//! info!(kvs: &ctx, "something to log");
//! ```
use log::kv::{Error, Key, Value, Visitor};
use log::{Log, Metadata, Record, SetLoggerError};
use std::io::Write;
use std::net::UdpSocket;

// publicly exporting so $crate::Level works.
pub use log::Level;

// publicly exporting to make configuring simpler.
pub use log::LevelFilter;

/// The statically resolved maximum log level.
pub const STATIC_MAX_LEVEL: LevelFilter = log::STATIC_MAX_LEVEL;

/// Returns the current maximum log level.
#[inline]
pub fn max_level() -> LevelFilter {
    log::max_level()
}

/// The standard logging macro.
///
/// # Examples
///
/// ```no_run
/// # use std::vec::Vec;
/// use udp_logger_rs::info;
///
/// // The standard logging we know and love
/// info!("hello");
/// info!("hello",);
/// info!("hello {}", "cats");
/// info!("hello {}", "cats",);
/// info!(target: "MyApp", "hello");
/// info!(target: "MyApp", "hello",);
/// info!(target: "MyApp", "hello {}", "cats");
/// info!(target: "MyApp", "hello {}", "cats",);
///
/// // The kv logging we hope to love
/// let ctx: Vec<(String, String)> = vec![
///    ("cat_1".into(), "chashu".into()),
///    ("cat_2".into(), "nori".into()),
/// ];
///
/// // default target
/// info!(kvs: &ctx, "hello");
/// info!(kvs: &ctx, "hello",);
/// info!(kvs: &ctx, "hello {}", "cats");
/// info!(kvs: &ctx, "hello {}", "cats",);
///
/// // with target provided
/// info!(target: "MyApp", kvs: &ctx, "hello");
/// info!(target: "MyApp", kvs: &ctx, "hello",);
/// info!(target: "MyApp", kvs: &ctx, "hello {}", "cats");
/// info!(target: "MyApp", kvs: &ctx, "hello {}", "cats",);
/// ```
#[macro_export(local_inner_macros)]
macro_rules! log {
    (target: $target:expr, kvs: $kvs:expr, $lvl:expr, $($arg:tt)+) => ({
        let lvl = $lvl;
        if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
            $crate::__private_api_log(
                __log_format_args!($($arg)+),
                lvl,
                &($target, __log_module_path!(), __log_file!(), __log_line!()),
                Some($kvs),
            );
        }
    });
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
        let lvl = $lvl;
        if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
            $crate::__private_api_log(
                __log_format_args!($($arg)+),
                lvl,
                &($target, __log_module_path!(), __log_file!(), __log_line!()),
                None,
            );
        }
    });
    (kvs: $kvs:expr, $lvl:expr, $($arg:tt)+) => ({
        (log!(target: __log_module_path!(), kvs: $kvs, $lvl, $($arg)+))    });

    ($lvl:expr, $($arg:tt)+) => (log!(target: __log_module_path!(), $lvl, $($arg)+))
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! log_impl {
    // End of macro input
    (target: $target:expr, kvs: $kvs:expr, $lvl:expr, ($($arg:expr),*)) => {{
        let lvl = $lvl;
        if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
            $crate::__private_api_log(
                __log_format_args!($($arg),*),
                lvl,
                &($target, __log_module_path!(), __log_file!(), __log_line!()),
                $kvs,
            );
        }
    }};
}

/// Logs a message at the trace level.
#[macro_export(local_inner_macros)]
macro_rules! trace {
    (target: $target:expr, kvs: $kvs:expr, $($arg:tt)+) => (
        log!(target: $target, kvs: $kvs, $crate::Level::Trace, $($arg)+);
    );
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::Level::Trace, $($arg)+);
    );
    (kvs: $kvs:expr, $($arg:tt)+) => (
        log!(kvs: $kvs, $crate::Level::Trace, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!($crate::Level::Trace, $($arg)+);
    )
}

/// Logs a message at the debug level.
#[macro_export(local_inner_macros)]
macro_rules! debug {
    (target: $target:expr, kvs: $kvs:expr, $($arg:tt)+) => (
        log!(target: $target, kvs: $kvs, $crate::Level::Debug, $($arg)+);
    );
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::Level::Debug, $($arg)+);
    );
    (kvs: $kvs:expr, $($arg:tt)+) => (
        log!(kvs: $kvs, $crate::Level::Debug, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!($crate::Level::Debug, $($arg)+);
    )
}

/// Logs a message at the info level.
#[macro_export(local_inner_macros)]
macro_rules! info {
    (target: $target:expr, kvs: $kvs:expr, $($arg:tt)+) => (
        log!(target: $target, kvs: $kvs, $crate::Level::Info, $($arg)+);
    );
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::Level::Info, $($arg)+);
    );
    (kvs: $kvs:expr, $($arg:tt)+) => (
        log!(kvs: $kvs, $crate::Level::Info, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!($crate::Level::Info, $($arg)+);
    )
}

/// Logs a message at the warn level.
#[macro_export(local_inner_macros)]
macro_rules! warn {
    (target: $target:expr, kvs: $kvs:expr, $($arg:tt)+) => (
        log!(target: $target, kvs: $kvs, $crate::Level::Warn, $($arg)+);
    );
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::Level::Warn, $($arg)+);
    );
    (kvs: $kvs:expr, $($arg:tt)+) => (
        log!(kvs: $kvs, $crate::Level::Warn, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!($crate::Level::Warn, $($arg)+);
    )
}

/// Logs a message at the error level.
#[macro_export(local_inner_macros)]
macro_rules! error {
    (target: $target:expr, kvs: $kvs:expr, $($arg:tt)+) => (
        log!(target: $target, kvs: $kvs, $crate::Level::Error, $($arg)+);
    );
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::Level::Error, $($arg)+);
    );
    (kvs: $kvs:expr, $($arg:tt)+) => (
        log!(kvs: $kvs, $crate::Level::Error, $($arg)+);
    );
    ($($arg:tt)+) => (
        log!($crate::Level::Error, $($arg)+);
    )
}

/// Determines if a message logged at the specified level in that module will
/// be logged.
#[macro_export(local_inner_macros)]
macro_rules! log_enabled {
    (target: $target:expr, $lvl:expr) => {{
        let lvl = $lvl;
        lvl <= $crate::STATIC_MAX_LEVEL
            && lvl <= $crate::max_level()
            && $crate::__private_api_enabled(lvl, $target)
    }};
    ($lvl:expr) => {
        log_enabled!(target: __log_module_path!(), $lvl)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_format_args {
    ($($args:tt)*) => {
        format_args!($($args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_module_path {
    () => {
        module_path!()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_file {
    () => {
        file!()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_line {
    () => {
        line!()
    };
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
#[doc(hidden)]
pub fn __private_api_log(
    args: std::fmt::Arguments<'_>,
    level: log::Level,
    &(target, module_path, file, line): &(&str, &'static str, &'static str, u32),
    kvs: Option<&dyn log::kv::Source>,
) {
    log::logger().log(
        &log::Record::builder()
            .args(args)
            .level(level)
            .target(target)
            .module_path_static(Some(module_path))
            .file_static(Some(file))
            .line(Some(line))
            .key_values(&kvs)
            .build(),
    );
}

// enough with the macros, on with the UDP logging

/// Wire formats. Default is Uncompressed.
///
/// * Uncompressed, the entire payload is a string, formatted as:
/// ```no_run
/// # use chrono::Utc;
/// # let record = log::Record::builder().build();
/// # let target = "App";
/// format!("{} {:<5} [{}] {}",
///     Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
///     record.level().to_string(),
///     target,
///     record.args()
/// );
/// ```
/// and has kv pairs, appended, as:
/// ```no_run
/// # let k = "key1";
/// # let v = "value1";
/// format!(" {}={}", k, v);
/// ```
/// * ByteBuffer, the entire payload is a u8 level, i64 Utc::now().timestamp_millis(), and
/// u32 string length followed by length * utf8.
#[derive(Debug)]
pub enum WireFmt {
    /// No Compression, the payload can be consistered a string of utf8 bytes.
    Uncompressed,
    /// 1 byte Level, 8 bytes timestamp, 4 bytes len followed by len * utf8 (string)
    ByteBuffer,
}

/// The UdpLogger is a control structure for logging via UDP packets.
#[derive(Debug)]
pub struct UdpLogger {
    default_level: LevelFilter,
    module_levels: Vec<(String, LevelFilter)>,
    default_source: UdpSocket,
    sources: Vec<(LevelFilter, UdpSocket)>,
    default_destination: String,
    destinations: Vec<(LevelFilter, String)>,
    wire_fmt: WireFmt,
}

impl UdpLogger {
    /// Initializes the global logger with a UdpLogger instance with
    /// default log level set to `Level::Trace`.
    ///
    /// # Examples
    /// ```no_run
    /// use udp_logger_rs::{UdpLogger, warn};
    ///
    /// UdpLogger::new().env().init().unwrap();
    /// warn!("This is an example message.");
    /// ```
    ///
    /// [`init`]: #method.init
    #[must_use = "You must call init() to begin logging"]
    pub fn new() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:4000").expect("unable to bind to socket");
        socket
            .set_nonblocking(true)
            .expect("unable to set socket non-blocking");

        Self {
            default_level: LevelFilter::Trace,
            module_levels: Vec::new(),
            default_source: socket,
            sources: Vec::new(),
            default_destination: "127.0.0.1:4010".to_string(),
            destinations: Vec::new(),
            wire_fmt: WireFmt::Uncompressed,
        }
    }

    /// Simulates env_logger behavior, which enables the user to choose log
    /// level by setting a `RUST_LOG` environment variable. This will use
    /// the default level set by [`with_level`] if `RUST_LOG` is not set or
    /// can't be parsed as a standard log level.
    ///
    /// [`with_level`]: #method.with_level
    #[must_use = "You must call init() to begin logging"]
    pub fn env(mut self) -> Self {
        if let Ok(level) = std::env::var("RUST_LOG") {
            match level.to_lowercase().as_str() {
                "trace" => self.default_level = log::LevelFilter::Trace,
                "debug" => self.default_level = log::LevelFilter::Debug,
                "info" => self.default_level = log::LevelFilter::Info,
                "warn" => self.default_level = log::LevelFilter::Warn,
                "error" => self.default_level = log::LevelFilter::Error,
                _ => (),
            }
        };
        self
    }

    /// Set the 'default' log level.
    ///
    /// You can override the default level for specific modules and their sub-modules using [`with_module_level`]
    ///
    /// [`with_module_level`]: #method.with_module_level
    #[must_use = "You must call init() to begin logging"]
    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.default_level = level;
        self
    }

    /// Override the log level for some specific modules.
    ///
    /// This sets the log level of a specific module and all its sub-modules.
    /// When both the level for a parent module as well as a child module are set,
    /// the more specific value is taken. If the log level for the same module is
    /// specified twice, the resulting log level is implementation defined.
    ///
    /// # Examples
    ///
    /// Silence an overly verbose crate:
    ///
    /// ```no_run
    /// use udp_logger_rs::UdpLogger;
    /// use log::LevelFilter;
    ///
    /// UdpLogger::new().with_module_level("chatty_dependency", LevelFilter::Warn).init().unwrap();
    /// ```
    ///
    /// Disable logging for all dependencies:
    ///
    /// ```no_run
    /// use udp_logger_rs::UdpLogger;
    /// use log::LevelFilter;
    ///
    /// UdpLogger::new()
    ///     .with_level(LevelFilter::Off)
    ///     .with_module_level("my_crate", LevelFilter::Info)
    ///     .init()
    ///     .unwrap();
    /// ```
    #[must_use = "You must call init() to begin logging"]
    pub fn with_module_level(mut self, target: &str, level: LevelFilter) -> Self {
        self.module_levels.push((target.to_string(), level));

        /* Normally this is only called in `init` to avoid redundancy, but we can't initialize the logger in tests */
        #[cfg(test)]
        self.module_levels
            .sort_by_key(|(name, _level)| name.len().wrapping_neg());

        self
    }

    /// Override the default source socket.
    ///
    /// This sets the default source socket, which otherwise defaults to "127.0.0.1:4000".
    ///
    /// # Examples
    ///
    /// Log from UDP port "127.0.0.1:4444"
    ///
    /// ```no_run
    /// use udp_logger_rs::UdpLogger;
    ///
    /// UdpLogger::new()
    ///     .with_source("127.0.0.1:4444")
    ///     .init()
    ///     .unwrap();
    /// ```
    #[must_use = "You must call init() to begin logging"]
    pub fn with_source(mut self, source: &str) -> Self {
        let socket = UdpSocket::bind(source).expect("unable to bind to socket");
        socket
            .set_nonblocking(true)
            .expect("unable to set socket non-blocking");
        self.default_source = socket;

        self
    }

    /// Provide a level specific source address.
    ///
    /// This sets the source address, for log messages matching the level.
    ///
    /// # Examples
    ///
    /// Log from UDP port "127.0.0.1:4001" all Info log messages. Trace and Debug messages
    /// will log from the default source address. If with_source_level() for
    /// Warn or Error aren't set, those levels will also send from "127.0.0.1:4001".
    ///
    /// ```no_run
    /// use udp_logger_rs::UdpLogger;
    /// use log::LevelFilter;
    ///
    /// UdpLogger::new()
    ///     .with_source_level("127.0.0.1:4001", LevelFilter::Info)
    ///     .init()
    ///     .unwrap();
    /// ```
    #[must_use = "You must call init() to begin logging"]
    pub fn with_source_level(mut self, source: &str, level: LevelFilter) -> Self {
        let socket = UdpSocket::bind(source).expect("unable to bind to socket");
        socket
            .set_nonblocking(true)
            .expect("unable to set socket non-blocking");
        self.sources.push((level, socket));

        self
    }

    /// Override the default destination address.
    ///
    /// This sets the default destination address, which otherwise defaults to "127.0.0.1:4010".
    ///
    /// # Examples
    ///
    /// Log to UDP port "127.0.0.1:4040"
    ///
    /// ```no_run
    /// use udp_logger_rs::UdpLogger;
    ///
    /// UdpLogger::new()
    ///     .with_destination("127.0.0.1:4040")
    ///     .init()
    ///     .unwrap();
    /// ```
    #[must_use = "You must call init() to begin logging"]
    pub fn with_destination(mut self, destination: &str) -> Self {
        self.default_destination = destination.to_string();

        self
    }

    /// Provide a level specific destination address.
    ///
    /// This sets the destination address, for log messages matching the level.
    ///
    /// # Examples
    ///
    /// Log to UDP port "127.0.0.1:4040" all Info log messages. Trace and Debug messages
    /// will send to the default destination address. If with_destination_level() for
    /// Warn or Error aren't set, those levels will also send to "127.0.0.1:4040".
    ///
    /// ```no_run
    /// use udp_logger_rs::UdpLogger;
    /// use log::LevelFilter;
    /// UdpLogger::new()
    ///     .with_destination_level("127.0.0.1:4040", LevelFilter::Info)
    ///     .init()
    ///     .unwrap();
    /// ```
    #[must_use = "You must call init() to begin logging"]
    pub fn with_destination_level(mut self, destination: &str, level: LevelFilter) -> Self {
        self.destinations.push((level, destination.to_string()));

        self
    }

    /// Set the wire format for logging.
    #[must_use = "You must call init() to begin logging"]
    pub fn with_wire_fmt(mut self, wire_fmt: WireFmt) -> Self {
        self.wire_fmt = wire_fmt;

        self
    }

    #[doc(hidden)]
    // partial_init is used internally in init() and in testing.
    pub fn partial_init(mut self) -> Self {
        /* Sort all module levels from most specific to least specific. The length of the module
         * name is used instead of its actual depth to avoid module name parsing.
         */
        self.module_levels
            .sort_by_key(|(name, _level)| name.len().wrapping_neg());
        let max_level = self
            .module_levels
            .iter()
            .map(|(_name, level)| level)
            .copied()
            .max();
        let max_level = max_level
            .map(|lvl| lvl.max(self.default_level))
            .unwrap_or(self.default_level);

        self.sources.sort_by_key(|(level, _socket)| *level);
        self.destinations.sort_by_key(|(level, _socket)| *level);
        log::set_max_level(max_level);

        self
    }
    /// 'Init' the actual logger, instantiate it and configure it,
    /// this method MUST be called in order for the logger to be effective.
    pub fn init(self) -> Result<(), SetLoggerError> {
        let logger = self.partial_init();
        log::set_boxed_logger(Box::new(logger))?;
        Ok(())
    }
}

impl Default for UdpLogger {
    /// See [this](struct.UdpLogger.html#method.new)
    fn default() -> Self {
        UdpLogger::new()
    }
}

#[derive(Default)]
struct KVAccumulator(String);

impl<'kvs> Visitor<'kvs> for KVAccumulator {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        self.0.push_str(&format!(" {}={}", key, value));
        Ok(())
    }
}

impl Log for UdpLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        &metadata.level().to_level_filter()
            <= self
                .module_levels
                .iter()
                /* At this point the Vec is already sorted so that we can simply take
                 * the first match
                 */
                .find(|(name, _level)| metadata.target().starts_with(name))
                .map(|(_name, level)| level)
                .unwrap_or(&self.default_level)
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let socket = self
                .sources
                .iter()
                .find(|(level, _socket)| level >= &record.level())
                .map(|(_level, socket)| socket)
                .unwrap_or_else(|| &self.default_source);

            let remote_addr = self
                .destinations
                .iter()
                .find(|(level, _socket)| level >= &record.level())
                .map(|(_level, socket)| socket)
                .unwrap_or_else(|| &self.default_destination);

            let target = if !record.target().is_empty() {
                record.target()
            } else {
                record.module_path().unwrap_or_default()
            };
            let source = record.key_values();
            let mut visitor = KVAccumulator::default();
            let _result = source.visit(&mut visitor);

            let result = match self.wire_fmt {
                WireFmt::Uncompressed => {
                    let payload = format!(
                        "{} {:<5} [{}] {}{}",
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                        record.level().to_string(),
                        target,
                        record.args(),
                        visitor.0
                    );
                    socket.send_to(payload.as_bytes(), remote_addr)
                }
                WireFmt::ByteBuffer => {
                    let mut encoder = bytebuffer::ByteBuffer::new();
                    let level: [u8; 1] = match record.level() {
                        Level::Error => [1],
                        Level::Warn => [2],
                        Level::Info => [3],
                        Level::Debug => [4],
                        Level::Trace => [5],
                    };
                    let now = chrono::Utc::now().timestamp_millis().to_be_bytes();
                    let text = format!("[{}] {}{}", target, record.args(), visitor.0);
                    encoder
                        .write(&level)
                        .and_then(|_count| encoder.write(&now))
                        .and_then(|_count| {
                            encoder.write_string(&text);
                            socket.send_to(&encoder.to_bytes(), remote_addr)
                        })
                }
            };
            match result {
                Ok(_) => (),
                Err(err) => {
                    println!("error sending payload, err={}", err)
                }
            };
        }
    }

    fn flush(&self) {}
}
