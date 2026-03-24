//! Error types for DA commitment operations.

#[derive(Debug)]
pub enum Error {
    InvalidConfig(&'static str),
    InvalidProof,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidConfig(msg) => write!(f, "invalid config: {msg}"),
            Self::InvalidProof => write!(f, "invalid proof"),
        }
    }
}
