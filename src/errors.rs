//! Definition of errors.
use std::{fmt, result};

/// A specialized Result type for Crawdad.
pub type Result<T, E = CrawdadError> = result::Result<T, E>;

/// Errors in crawdad.
#[derive(Debug)]
pub enum CrawdadError {
    /// Contains [`InputError`].
    Input(InputError),

    /// Contains [`SetupError`].
    Setup(SetupError),

    /// Contains [`ScaleError`].
    Scale(ScaleError),
}

impl fmt::Display for CrawdadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Input(e) => e.fmt(f),
            Self::Setup(e) => e.fmt(f),
            Self::Scale(e) => e.fmt(f),
        }
    }
}

impl CrawdadError {
    pub(crate) const fn input(msg: &'static str) -> Self {
        Self::Input(InputError { msg })
    }
    pub(crate) const fn setup(msg: &'static str) -> Self {
        Self::Setup(SetupError { msg })
    }
    pub(crate) const fn scale(arg: &'static str, max: u32) -> Self {
        Self::Scale(ScaleError { arg, max })
    }
}

/// Error used when the input argument is invalid.
#[derive(Debug)]
pub struct InputError {
    msg: &'static str,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InputError: {}", self.msg)
    }
}

/// Error used when the setup is invalid.
#[derive(Debug)]
pub struct SetupError {
    msg: &'static str,
}

impl fmt::Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SetupError: {}", self.msg)
    }
}

/// Error used when the scale of a resulting trie exceeds the expected one.
#[derive(Debug)]
pub struct ScaleError {
    arg: &'static str,
    max: u32,
}

impl fmt::Display for ScaleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ScaleError: {} must be no greater than {}",
            self.arg, self.max
        )
    }
}
