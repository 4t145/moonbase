/// Data that can be load from config file, environment variables, config center, or other sources.

#[derive(Debug, Clone, Default)]
pub struct Config<T>(T);
