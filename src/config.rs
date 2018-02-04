//! Flower id configuration
//! See /README.md for addotation information
//! *WARNING* if you edit this parametrs rustdoc-test will been broken
//! *WARNING* don't forget edit self::test_constants::* for successful passing of tests

/// Genrator length (in bits)
pub const GENERATOR_LENGTH: u16 = 10;
/// Sequence length (in bits)
pub const SEQUENCE_LENGTH: u16 = 11;
/// Timestamp length (in bits)
pub const TIMESTAMP_LENGTH: u64 = 42;

/// Default `FIDGeneratorBuilder` values
/// All values can be modify in runtime see `FIDGeneratorBuilder`
pub(crate) mod gbuilder_defaults {
    /// Default unix timestamp offset (in seconds)
    pub const TIMESTAMP_OFFSET: i64 = -1483228800;
    /// Timestamp in seconds?
    pub const TIMESTAMP_IN_SECONDS: bool = false;
    /// Wait next timestamp if sequence is overflowed
    pub const WAIT_SEQUENCE: bool = true;
}

/// Contants for tests
#[cfg(test)]
#[cfg(test)]
pub(crate) mod test_constants {
    pub const GENERATOR: u16 = 0x01cc_u16;
    pub const SEQUENCE: u16 = 0x02f8_u16;
    pub const TIMESTAMP: u64 = 0x1f37b5bfdfa_u64;
    pub const BIN: &[u8; 8] = b">ok\x7f\xbfK\xe1\xcc";
    pub const B64: &[u8; 11] = b"Pm9rf79L4cw";
}

// Don't change next constants
pub const GENERATOR_MASK: u64 = ((1 << GENERATOR_LENGTH) - 1);
pub const SEQUENCE_MASK: u64 = ((1 << SEQUENCE_LENGTH) - 1) << GENERATOR_LENGTH;
pub const TIMESTAMP_MASK: u64 =
    ((1 << TIMESTAMP_LENGTH) - 1) << (GENERATOR_LENGTH + SEQUENCE_LENGTH);
