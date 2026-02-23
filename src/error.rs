use crate::time_duration;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    TimeDuration(#[from] time_duration::ParseError),
    #[error("Invalid breakdown value: {0}")]
    InvalidBreakDown(String),
}
