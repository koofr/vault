pub fn setup_logging() {
    let mut env_logger_builder = env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let _ = env_logger_builder.try_init();
}
