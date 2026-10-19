// SPDX-License-Identifier: LGPL-2.1-or-later
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    Budget(&'static str),
    Overflow(&'static str),
    Missing(&'static str),
    Stale(&'static str),
    Permission(&'static str),
    Unsupported(&'static str),
    Canceled,
    Busy,
    WrongOwner,
    Io(String),
}
pub type Result<T> = std::result::Result<T, Error>;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(s) => write!(f, "invalid {s}"),
            Self::Budget(s) => write!(f, "budget exceeded: {s}"),
            Self::Overflow(s) => write!(f, "overflow: {s}"),
            Self::Missing(s) => write!(f, "missing {s}"),
            Self::Stale(s) => write!(f, "stale {s}"),
            Self::Permission(s) => write!(f, "permission denied: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported {s}"),
            Self::Canceled => f.write_str("canceled"),
            Self::Busy => f.write_str("publication barrier active"),
            Self::WrongOwner => f.write_str("wrong transaction owner"),
            Self::Io(s) => write!(f, "I/O: {s}"),
        }
    }
}
impl std::error::Error for Error {}
