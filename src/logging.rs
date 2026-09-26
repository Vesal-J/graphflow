use std::io::Write;
use env_logger::{Builder, Env};
pub use log::LevelFilter;
pub use log::{debug, error, info, trace, warn};

/// Initializes the Graphflow logging system with formatted, colored output.
///
/// Respects the `RUST_LOG` environment variable if set. If unset, it defaults to the `INFO` log level.
/// If a logger has already been initialized in the process, this call is a safe no-op.
pub fn init() {
    let _ = try_init();
}

/// Initializes the logging system with a specific default `LevelFilter`.
///
/// If `RUST_LOG` is explicitly set in the environment, `RUST_LOG` takes precedence.
pub fn init_with_level(default_level: LevelFilter) {
    let _ = try_init_with_level(default_level);
}

/// Attempts to initialize the Graphflow logger, returning an error if a logger has already been registered.
pub fn try_init() -> Result<(), log::SetLoggerError> {
    try_init_with_level(LevelFilter::Info)
}

/// Attempts to initialize the logger with a custom default `LevelFilter`.
pub fn try_init_with_level(default_level: LevelFilter) -> Result<(), log::SetLoggerError> {
    let env = Env::default().default_filter_or(default_level.to_string());
    let mut builder = Builder::from_env(env);

    builder.format(|buf, record| {
        let level_style = buf.default_level_style(record.level());
        let ts = buf.timestamp();
        writeln!(
            buf,
            "[{ts} {level_style}{:<5}{level_style:#} {}] {}",
            record.level(),
            record.target(),
            record.args()
        )
    });

    builder.try_init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_try_init() {
        // init() should not panic regardless of logger state
        init();

        // After logger is initialized in the process, try_init should return Err
        let res = try_init();
        assert!(res.is_err(), "Expected try_init() to return Err when logger is already registered");
    }

    #[test]
    fn test_init_with_level_and_try_init_with_level() {
        // init_with_level should be a safe no-op if logger is already set
        init_with_level(LevelFilter::Debug);
        init_with_level(LevelFilter::Trace);

        // try_init_with_level should return Err when logger is already set
        let res = try_init_with_level(LevelFilter::Warn);
        assert!(res.is_err(), "Expected try_init_with_level to return Err when logger is already registered");
    }

    #[test]
    fn test_logging_reexports() {
        // Test level filter enum values
        assert_ne!(LevelFilter::Off, LevelFilter::Info);
        assert_ne!(LevelFilter::Debug, LevelFilter::Error);

        // Test log macros re-exported from logging module
        info!("Testing re-exported info macro");
        warn!("Testing re-exported warn macro");
        error!("Testing re-exported error macro");
        debug!("Testing re-exported debug macro");
        trace!("Testing re-exported trace macro");
    }
}

