use std::{fmt, result};

/// A specialized Result type for Crawdad.
pub type Result<T, E = CrawdadError> = result::Result<T, E>;

#[derive(Debug)]
pub enum CrawdadError {
    Input(InputError),
    Setup(SetupError),
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

#[derive(Debug)]
pub struct InputError {
    msg: &'static str,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InputError: {}", self.msg)
    }
}

#[derive(Debug)]
pub struct SetupError {
    msg: &'static str,
}

impl fmt::Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SetupError: {}", self.msg)
    }
}

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
