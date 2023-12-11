use std::{
    io::{stderr, stdout},
    sync::Once,
};
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use crate::formatter::MXFormatter;
use super::*;

static INIT: Once = Once::new();

// initialize a logger with the value of the given environment variable

#[inline]
pub fn init_logger(name: &str) {
    let name = name.replace('-', "_");
    let env_filter = EnvFilter::try_from_env(name.to_uppercase() + "_LOG").unwrap_or_else(|_| EnvFilter::new(name + "=info"));
    init_logger_with_env_filter(env_filter, false);
}

#[inline]
pub fn init_logger_with(directives: &str, colorful: bool) {
    let env_filter = EnvFilter::new(directives.replace('-', "_"));
    init_logger_with_env_filter(env_filter, colorful);
}

fn init_logger_with_env_filter(env_filter: EnvFilter, colorful: bool) {
    let writer = stderr.with_max_level(Level::ERROR).or_else(stdout);
    INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .map_writer(move |_| writer)
                    .map_event_format(|_| MXFormatter { colorful }),
            )
            .with(env_filter)
            .init()
    });
}
