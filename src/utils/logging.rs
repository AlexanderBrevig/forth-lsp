//! Logging utilities for LSP handlers

/// Log a request with its ID and parameters
#[macro_export]
macro_rules! log_request {
    ($id:expr, $params:expr) => {
        eprintln!("#{}: {:?}", $id, $params);
    };
}

/// Log a request with a custom message
#[macro_export]
macro_rules! log_request_msg {
    ($id:expr, $msg:expr) => {
        eprintln!("#{}: {}", $id, $msg);
    };
    ($id:expr, $fmt:expr, $($arg:tt)*) => {
        eprintln!("#{}: {}", $id, format!($fmt, $($arg)*));
    };
}

/// Log a handler error
#[macro_export]
macro_rules! log_handler_error {
    ($handler:expr, $err:expr) => {
        eprintln!("{} handler error: {:?}", $handler, $err);
    };
}

/// Log a debug message
#[macro_export]
macro_rules! log_debug {
    ($msg:expr) => {
        eprintln!("{}", $msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!($fmt, $($arg)*);
    };
}
