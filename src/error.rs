use macron::{ Display, From, Error };

/// Std Result alias
pub type StdResult<T, E> = std::result::Result<T, E>;
/// Result alias
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// Application error
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[from]
    String(String),

    #[from]
    Logger(log::SetLoggerError),

    #[display = "Failed to read '{0}.bat' file"]
    FailedParseBatFile(String),
}
