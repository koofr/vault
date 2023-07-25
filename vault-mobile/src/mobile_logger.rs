use log::{Level, Log, Metadata, Record, SetLoggerError};

use crate::{LoggerCallback, LoggerLevel};

struct MobileLogger {
    cb: Box<dyn LoggerCallback>,
}

impl Log for MobileLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        self.cb
            .log(record.level().into(), format!("{}", record.args()))
    }

    fn flush(&self) {}
}

pub fn init_with_level(
    level: LoggerLevel,
    cb: Box<dyn LoggerCallback>,
) -> Result<(), SetLoggerError> {
    let level: Level = level.into();

    log::set_boxed_logger(Box::new(MobileLogger { cb }))?;

    log::set_max_level(level.to_level_filter());

    Ok(())
}

pub fn try_init_env_logger() {
    let mut env_logger_builder = env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    env_logger_builder.filter_module("vault_core", log::LevelFilter::Debug);
    env_logger_builder.filter_module("vault_mobile", log::LevelFilter::Debug);
    env_logger_builder.filter_module("vault_native", log::LevelFilter::Debug);
    env_logger_builder.filter_module("vault_fake_remote", log::LevelFilter::Debug);

    let _ = env_logger_builder.try_init();
}
