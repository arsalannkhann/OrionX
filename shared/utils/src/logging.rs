use anyhow::Result;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

use crate::config::LoggingConfig;

pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = tracing_subscriber::registry().with(env_filter);

    match config.format.as_str() {
        "json" => {
            let fmt_layer = fmt::layer()
                .json()
                .with_span_events(FmtSpan::CLOSE)
                .with_thread_ids(true)
                .with_thread_names(true);

            if let Some(file_path) = &config.file_path {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)?;
                
                registry
                    .with(fmt_layer.with_writer(file))
                    .init();
            } else {
                registry
                    .with(fmt_layer)
                    .init();
            }
        }
        _ => {
            let fmt_layer = fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_thread_ids(true)
                .with_thread_names(true);

            if let Some(file_path) = &config.file_path {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)?;
                
                registry
                    .with(fmt_layer.with_writer(file))
                    .init();
            } else {
                registry
                    .with(fmt_layer)
                    .init();
            }
        }
    }

    tracing::info!("Logging initialized with level: {}", config.level);
    Ok(())
}

#[macro_export]
macro_rules! log_error {
    ($err:expr, $msg:expr) => {
        tracing::error!(error = %$err, $msg);
    };
    ($err:expr, $msg:expr, $($field:tt)*) => {
        tracing::error!(error = %$err, $msg, $($field)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($msg:expr) => {
        tracing::warn!($msg);
    };
    ($msg:expr, $($field:tt)*) => {
        tracing::warn!($msg, $($field)*);
    };
}

#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        tracing::info!($msg);
    };
    ($msg:expr, $($field:tt)*) => {
        tracing::info!($msg, $($field)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($msg:expr) => {
        tracing::debug!($msg);
    };
    ($msg:expr, $($field:tt)*) => {
        tracing::debug!($msg, $($field)*);
    };
}