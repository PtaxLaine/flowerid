pub mod id;
pub mod generator;
pub mod base64;
mod limits;
#[cfg(test)]
mod stub_time_systemtime;

use std::result;
use std::fmt;
use std::error;

/// Errors
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    /// timestamp overflow
    TimestampOverflow(u64),
    /// sequence overflow
    SequenceOverflow(u16),
    /// generator overflow
    GeneratorOverflow(u16),
    /// system time is in past
    SysTimeIsInPast,
    /// slice length not eq 8 bytes
    WrongSliceSize(usize),
    /// padding error
    Base64PaddingError,
    /// bad symbol error
    Base64WrongSymbolError,
    /// (only ignore mode) combine Padding & WrongSymbol
    Base64PaddingWrongSymbolError,
}
pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "base64 decoding error"
    }
}
