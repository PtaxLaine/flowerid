//! Flower identificator

use std::fmt;
use base64;
use limits as lim;

use {Error, Result};

/// Flower identificator struct
#[derive(PartialEq, PartialOrd, Clone)]
pub struct FID(u64);

impl FID {
    /// Create FID from components
    /// timestamp - 42 bits value
    /// sequence  - 11 bits value
    /// generator - 10 bits value
    ///
    /// # Failures
    /// Error::{TimestampOverflow,SequenceOverflow,GeneratorOverflow}
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
        if timestamp >= 1 << lim::TIMESTAMP_LENGTH {
            Err(Error::TimestampOverflow(timestamp))
        } else if sequence >= 1 << lim::SEQUENCE_LENGTH {
            Err(Error::SequenceOverflow(sequence))
        } else if generator >= 1 << lim::GENERATOR_LENGTH {
            Err(Error::GeneratorOverflow(generator))
        } else {
            Ok(FID(
                (timestamp << (lim::SEQUENCE_LENGTH + lim::GENERATOR_LENGTH))
                    | ((sequence as u64) << lim::GENERATOR_LENGTH)
                    | (generator as u64),
            ))
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
        let tmp = self.0.to_be();
        unsafe {
            let p = &tmp as *const u64 as *const u8;
            [
                *p.offset(0),
                *p.offset(1),
                *p.offset(2),
                *p.offset(3),
                *p.offset(4),
                *p.offset(5),
                *p.offset(6),
                *p.offset(7),
            ]
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
        let mut tmp = 0u64;
        unsafe {
            let p = &mut tmp as *mut u64 as *mut u8;
            for i in 0..8usize {
                *p.offset(i as isize) = val[i];
            }
        }
        FID(u64::from_be(tmp))
    }

    /// Deserialize FID
    ///
    /// # Failures
    /// Error::WrongSliceSize if slice length != 8
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
        let mut tmp = 0u64;
        unsafe {
            let p = &mut tmp as *mut u64 as *mut u8;
            for i in 0..8usize {
                *p.offset(i as isize) = val[i];
            }
        }
        Ok(FID(u64::from_be(tmp)))
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
        let vec = base64::encode_without_pading(&self.to_bytes());
        [
            vec[0], vec[1], vec[2], vec[3], vec[4], vec[5], vec[6], vec[7], vec[8], vec[9], vec[10]
        ]
    }

    /// Deserialize FID from base64
    ///
    /// # Failures
    /// Error::WrongSliceSize if decoded length != 8
    /// Error::Base64WrongSymbolError decoding failed
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
        let vec = base64::decode(val, Some(Error::Base64PaddingError))?;
        FID::from_slice(&vec)
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
        (self.0 & lim::TIMESTAMP_MASK) >> (lim::GENERATOR_LENGTH + lim::SEQUENCE_LENGTH)
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
        ((self.0 & lim::SEQUENCE_MASK) >> lim::GENERATOR_LENGTH) as u16
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
        (self.0 & lim::GENERATOR_MASK) as u16
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_getters() {
        let timestamp = 0x1f37b5bfdfa_u64;
        let sequence = 0x02f8_u16;
        let generator = 0x01cc_u16;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        assert_eq!(fid.timestamp(), timestamp);
        assert_eq!(fid.sequence(), sequence);
        assert_eq!(fid.generator(), generator);
        assert_eq!(
            FID::new(1 << 42, 0, 0).unwrap_err(),
            Error::TimestampOverflow(1 << 42)
        );
        assert_eq!(
            FID::new(0, 1 << 11, 0).unwrap_err(),
            Error::SequenceOverflow(1 << 11)
        );
        assert_eq!(
            FID::new(0, 0, 1 << 10).unwrap_err(),
            Error::GeneratorOverflow(1 << 10)
        );
    }

    #[test]
    fn bytes() {
        let timestamp = 0x1f37b5bfdfa_u64;
        let sequence = 0x02f8_u16;
        let generator = 0x01cc_u16;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        let bytes = fid.to_bytes();
        let fid_from_bytes = FID::from_bytes(&bytes);
        let fid_from_slice = FID::from_slice(&bytes).unwrap();
        assert_eq!(&&bytes, &b">ok\x7f\xbfK\xe1\xcc");
        assert_eq!(fid, fid_from_bytes);
        assert_eq!(fid, fid_from_slice);
    }

    #[test]
    fn base64() {
        let timestamp = 0x1f37b5bfdfa_u64;
        let sequence = 0x02f8_u16;
        let generator = 0x01cc_u16;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        assert_eq!(&&fid.to_b64(), &b"Pm9rf79L4cw");
        assert_eq!(FID::from_b64(&fid.to_b64()).unwrap(), fid);
    }

    #[test]
    fn fmt() {
        let timestamp = 0x1f37b5bfdfa_u64;
        let sequence = 0x02f8_u16;
        let generator = 0x01cc_u16;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        assert_eq!(format!("{}", fid), "Pm9rf79L4cw");
        assert_eq!(
            format!("{:?}", fid),
            "FID{ id: \"Pm9rf79L4cw\"; ts: 2145258307066; seq: 760; gen: 460 }"
        );
    }

    #[test]
    fn from() {
        let timestamp = 0x1f37b5bfdfa_u64;
        let sequence = 0x02f8_u16;
        let generator = 0x01cc_u16;
        let fid = FID::new(timestamp, sequence, generator).unwrap();
        let from_fid: u64 = From::from(fid.clone());
        let from_64: FID = From::from(from_fid);
        assert_eq!(from_64, fid);
    }
}
