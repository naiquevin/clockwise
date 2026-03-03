use crate::time_duration;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    TimeDuration(#[from] time_duration::ParseError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
