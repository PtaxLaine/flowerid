//! Flower identificator

use base64;
use config as cfg;
use std;
use std::fmt;
use std::mem;

use {Error, Result};

/// Flower identificator struct
#[derive(PartialEq, PartialOrd, Clone)]
pub struct FID(u64);

impl FID {
    /// Create FID from components
    ///
    /// timestamp - 42 bits value
    /// sequence  - 11 bits value
    /// generator - 10 bits value
    ///
    /// # Failures
    /// `Error::TimestampOverflow`
    /// `Error::SequenceOverflow`
    /// `Error::GeneratorOverflow`
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::new(0x204dc595637, 0x4ac, 0x12c).unwrap();
    /// assert_eq!(
    ///     format!("{}", fid),
    ///     "QJuLKsbysSw"
    /// );
    /// ```
    pub fn new(timestamp: u64, sequence: u16, generator: u16) -> Result<FID> {
        if timestamp >= 1 << cfg::TIMESTAMP_LENGTH {
            Err(Error::TimestampOverflow(timestamp))
        } else if sequence >= 1 << cfg::SEQUENCE_LENGTH {
            Err(Error::SequenceOverflow(sequence))
        } else if generator >= 1 << cfg::GENERATOR_LENGTH {
            Err(Error::GeneratorOverflow(generator))
        } else {
            Ok(FID((timestamp
                << (cfg::SEQUENCE_LENGTH + cfg::GENERATOR_LENGTH))
                | ((sequence as u64) << cfg::GENERATOR_LENGTH)
                | (generator as u64)))
        }
    }

    /// Serialize FID
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::new(0x204dc595637, 0x4ac, 0x12c).unwrap();
    /// assert_eq!(
    ///     &fid.to_bytes(),
    ///     b"@\x9b\x8b*\xc6\xf2\xb1,"
    /// );
    /// ```
    pub fn to_bytes(&self) -> [u8; 8] {
        unsafe {
            let mut res: [u8; 8] = mem::uninitialized();
            let ptr = res.as_mut_ptr() as *mut u64;
            *ptr = self.0.to_be();
            return res;
        }
    }

    /// Deserialize FID
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::from_bytes(&b"@\x9b\x8b*\xc6\xf2\xb1,");
    /// assert_eq!(
    ///     format!("{}", fid),
    ///     "QJuLKsbysSw"
    /// );
    /// ```
    pub fn from_bytes(val: &[u8; 8]) -> FID {
        FID::from_slice(val).unwrap()
    }

    /// Deserialize FID
    ///
    /// # Failures
    /// `Error::WrongSliceSize` if slice length != 8
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let val = b"@\x9b\x8b*\xc6\xf2\xb1,";
    /// let fid = FID::from_slice(val).unwrap();
    /// assert_eq!(
    ///     format!("{}", fid),
    ///     "QJuLKsbysSw"
    /// );
    /// ```
    pub fn from_slice(val: &[u8]) -> Result<FID> {
        if val.len() != 8 {
            return Err(Error::WrongSliceSize(val.len()));
        }
        unsafe {
            let tmp = *(val.as_ptr() as *const u64);
            Ok(FID(u64::from_be(tmp)))
        }
    }

    /// Serialize FID to base64 string
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::new(0x204dc595637, 0x4ac, 0x12c).unwrap();
    /// assert_eq!("QJuLKsbysSw", Into::<String>::into(fid.clone()));
    /// assert_eq!(
    ///     fid.to_string(),
    ///     "QJuLKsbysSw"
    /// );
    /// ```
    pub fn to_string(&self) -> String {
        let b64 = self.to_b64();
        std::str::from_utf8(&b64).unwrap().to_string()
    }

    /// Deserialize FID from base64 string
    ///
    /// # Failures
    /// `Error::WrongSliceSize` if decoded length != 8
    /// `Error::Base64WrongSymbolError` decoding failed
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::from_string("QJuLKsbysSw").unwrap();
    /// assert_eq!(
    ///     format!("{}", fid),
    ///     "QJuLKsbysSw"
    /// );
    /// ```
    pub fn from_string(val: &str) -> Result<FID> {
        let val = val.as_bytes();
        FID::from_b64(val)
    }

    /// Serialize FID to base64
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::new(0x204dc595637, 0x4ac, 0x12c).unwrap();
    /// assert_eq!(
    ///     &&fid.to_b64(),
    ///     &b"QJuLKsbysSw"
    /// );
    /// ```
    pub fn to_b64(&self) -> [u8; 11] {
        let mut buffer = [0u8; 11];
        base64::urlsafe_encode_without_pading_into(&self.to_bytes(), &mut buffer).unwrap();
        buffer
    }

    /// Deserialize FID from base64
    ///
    /// # Failures
    /// `Error::WrongSliceSize` if decoded length != 8
    /// `Error::Base64WrongSymbolError` decoding failed
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::from_b64(b"QJuLKsbysSw").unwrap();
    /// assert_eq!(
    ///     format!("{}", fid),
    ///     "QJuLKsbysSw"
    /// );
    /// ```
    pub fn from_b64(val: &[u8]) -> Result<FID> {
        let mut buffer = [0u8; 8];
        base64::decode_into(val, Some(Error::Base64PaddingError), &mut buffer)?;
        Ok(FID::from_bytes(&buffer))
    }

    /// timestamp getter
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::from_b64(b"QJuLKsbysSw").unwrap();
    /// assert_eq!(
    ///     fid.timestamp(),
    ///     0x204dc595637
    /// );
    /// ```
    pub fn timestamp(&self) -> u64 {
        (self.0 & cfg::TIMESTAMP_MASK) >> (cfg::GENERATOR_LENGTH + cfg::SEQUENCE_LENGTH)
    }

    /// sequence getter
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::from_b64(b"QJuLKsbysSw").unwrap();
    /// assert_eq!(
    ///     fid.sequence(),
    ///     0x4ac
    /// );
    /// ```
    pub fn sequence(&self) -> u16 {
        ((self.0 & cfg::SEQUENCE_MASK) >> cfg::GENERATOR_LENGTH) as u16
    }

    /// generator getter
    ///
    /// # Examples
    /// ```
    /// use flowerid::id::FID;
    /// let fid = FID::from_b64(b"QJuLKsbysSw").unwrap();
    /// assert_eq!(
    ///     fid.generator(),
    ///     0x12c
    /// );
    /// ```
    pub fn generator(&self) -> u16 {
        (self.0 & cfg::GENERATOR_MASK) as u16
    }
}

impl fmt::Debug for FID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "FID{{ id: \"{}\"; ts: {}; seq: {}; gen: {} }}",
            self,
            self.timestamp(),
            self.sequence(),
            self.generator()
        )
    }
}

impl fmt::Display for FID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = self.to_b64();
        let s = String::from_utf8_lossy(&v);
        write!(f, "{}", s)
    }
}

impl From<u64> for FID {
    fn from(id: u64) -> FID {
        FID(id)
    }
}

impl From<FID> for u64 {
    fn from(id: FID) -> u64 {
        id.0
    }
}

impl From<FID> for String {
    fn from(id: FID) -> String {
        id.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_getters() {
        let timestamp = cfg::test_constants::TIMESTAMP;
        let sequence = cfg::test_constants::SEQUENCE;
        let generator = cfg::test_constants::GENERATOR;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        assert_eq!(fid.timestamp(), timestamp);
        assert_eq!(fid.sequence(), sequence);
        assert_eq!(fid.generator(), generator);
        assert_eq!(
            FID::new(1 << cfg::TIMESTAMP_LENGTH, 0, 0).unwrap_err(),
            Error::TimestampOverflow(1 << cfg::TIMESTAMP_LENGTH)
        );
        assert_eq!(
            FID::new(0, 1 << cfg::SEQUENCE_LENGTH, 0).unwrap_err(),
            Error::SequenceOverflow(1 << cfg::SEQUENCE_LENGTH)
        );
        assert_eq!(
            FID::new(0, 0, 1 << cfg::GENERATOR_LENGTH).unwrap_err(),
            Error::GeneratorOverflow(1 << cfg::GENERATOR_LENGTH)
        );
    }

    #[test]
    fn bytes() {
        let timestamp = cfg::test_constants::TIMESTAMP;
        let sequence = cfg::test_constants::SEQUENCE;
        let generator = cfg::test_constants::GENERATOR;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        let bytes = fid.to_bytes();
        let fid_from_bytes = FID::from_bytes(&bytes);
        let fid_from_slice = FID::from_slice(&bytes).unwrap();
        assert_eq!(&&bytes, &cfg::test_constants::BIN);
        assert_eq!(fid, fid_from_bytes);
        assert_eq!(fid, fid_from_slice);
    }

    #[test]
    fn base64() {
        let timestamp = cfg::test_constants::TIMESTAMP;
        let sequence = cfg::test_constants::SEQUENCE;
        let generator = cfg::test_constants::GENERATOR;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        assert_eq!(&&fid.to_b64(), &cfg::test_constants::B64);
        assert_eq!(FID::from_b64(&fid.to_b64()).unwrap(), fid);
    }

    #[test]
    fn fmt() {
        let timestamp = cfg::test_constants::TIMESTAMP;
        let sequence = cfg::test_constants::SEQUENCE;
        let generator = cfg::test_constants::GENERATOR;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        assert_eq!(
            format!("{}", fid),
            String::from_utf8_lossy(cfg::test_constants::B64)
        );
        assert_eq!(
            format!("{:?}", fid),
            format!(
                "FID{{ id: \"{}\"; ts: {}; seq: {}; gen: {} }}",
                String::from_utf8_lossy(cfg::test_constants::B64),
                timestamp,
                sequence,
                generator
            )
        );
    }

    #[test]
    fn from() {
        let timestamp = cfg::test_constants::TIMESTAMP;
        let sequence = cfg::test_constants::SEQUENCE;
        let generator = cfg::test_constants::GENERATOR;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        let from_fid: u64 = From::from(fid.clone());
        let from_64: FID = From::from(from_fid);
        assert_eq!(from_64, fid);
    }
}
