//! see `/README.md` for additional information
//!
//! # Examples
//! ```
//! use flowerid::FID;
//! use flowerid::generator::FIDGeneratorBuilder;
//!
//! let mut gen = FIDGeneratorBuilder::new(0x12c).build().unwrap();
//! let fid: FID = gen.next().unwrap();
//! println!("{}", fid);
//! println!("{:?}", fid);
//! ```

pub mod id;
pub mod generator;
pub mod base64;
pub mod config;
mod stubs;

pub use id::FID;

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
    /// (en|de)code_into buffer too small
    Base64BufferTooSmall,
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
