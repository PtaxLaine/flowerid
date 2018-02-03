//! Base64 encoder/decoder

use std::fmt;
use std::error;

static ALPHABET: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];
static ALPHABET_SAFE: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'-', b'_',
];

/// Decode errors
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    /// padding error
    Padding,
    /// bad symbol error
    WrongSymbol,
    /// (only ignore mode) combine Padding & WrongSymbol
    PaddingWrongSymbol,
}

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

/// Encode bytes
/// # Examples
///
/// ```
/// use flowerid::base64::encode;
/// assert_eq!(encode(b"foo bar"), b"Zm9vIGJhcg==");
/// assert_eq!(encode(b"\xfb\xef\xff"), b"++//");
/// ```
pub fn encode(data: &[u8]) -> Vec<u8> {
    _encode(data, true, &ALPHABET)
}

/// Encode bytes URL safe
/// # Examples
///
/// ```
/// use flowerid::base64::urlsafe_encode;
/// assert_eq!(urlsafe_encode(b"foo bar"), b"Zm9vIGJhcg==");
/// assert_eq!(urlsafe_encode(b"\xfb\xef\xff"), b"--__");
/// ```
pub fn urlsafe_encode(data: &[u8]) -> Vec<u8> {
    _encode(data, true, &ALPHABET_SAFE)
}

/// Encode bytes without padding
/// # Examples
///
/// ```
/// use flowerid::base64::{encode,encode_without_pading};
/// assert_eq!(encode(b"foo bar"), b"Zm9vIGJhcg==");
/// assert_eq!(encode_without_pading(b"foo bar"), b"Zm9vIGJhcg");
/// ```
pub fn encode_without_pading(data: &[u8]) -> Vec<u8> {
    _encode(data, false, &ALPHABET)
}

/// Encode bytes URL safe without padding
/// # Examples
///
/// ```
/// use flowerid::base64::{urlsafe_encode,urlsafe_encode_without_pading};
/// assert_eq!(urlsafe_encode(b"foo bar"), b"Zm9vIGJhcg==");
/// assert_eq!(urlsafe_encode_without_pading(b"foo bar"), b"Zm9vIGJhcg");
/// ```
pub fn urlsafe_encode_without_pading(data: &[u8]) -> Vec<u8> {
    _encode(data, false, &ALPHABET_SAFE)
}

fn _encode(data: &[u8], padding: bool, aplhabet: &[u8; 64]) -> Vec<u8> {
    let mut result = Vec::<u8>::with_capacity(4 * (data.len() / 3));
    let mut i = 0usize;
    while i < data.len() {
        let mut group = 0u32;
        for j in 0..3 {
            if j + i >= data.len() {
                break;
            }
            group |= (data[j + i] as u32) << (16 - 8 * j);
        }
        for j in 0..4 {
            result.push(aplhabet[((group >> (18 - 6 * j)) & 0x3f) as usize]);
        }
        i += 3;
    }
    if data.len() % 3 != 0 {
        i = 0;
        while i < 24 - (data.len() * 8) % 24 {
            if padding {
                let rlen = result.len();
                result[rlen - 1 - i / 8] = b'=';
            } else {
                result.pop();
            }
            i += 8;
        }
    }
    result
}

/// Decode bytes
///
/// # Failures
/// Error::{Padding,WrongSymbol}
///
/// # Examples
///
/// ```
/// use flowerid::base64::{decode, Error};
/// assert_eq!(decode(b"Zm9vIGJhcg==", None).unwrap(), b"foo bar");
/// assert_eq!(decode(b"++//", None).unwrap(), b"\xfb\xef\xff");
/// assert_eq!(decode(b"--__", None).unwrap(), b"\xfb\xef\xff");
/// assert!(decode(b"Zm9vIGJhcg", None).is_err());
/// assert!(decode(b"Zm9vIGJhcg!", None).is_err());
/// assert_eq!(decode(b"Zm9vIGJhcg", Some(Error::Padding)).unwrap(), b"foo bar");
/// assert_eq!(decode(b"Zm9vIGJh!", Some(Error::WrongSymbol)).unwrap(), b"foo ba");
/// assert_eq!(decode(b"Zm9vIGJhcg!", Some(Error::PaddingWrongSymbol)).unwrap(), b"foo bar");
/// ```
pub fn decode(data: &[u8], ignore_error: Option<Error>) -> Result<Vec<u8>, Error> {
    let mut result = Vec::<u8>::with_capacity((data.len() / 4) * 3);
    let mut i = 0;
    while i < data.len() {
        let mut group = 0u32;
        let mut length = 0usize;
        let mut padding = 0usize;
        for j in 0..4 {
            if i + j >= data.len() {
                break;
            }
            if data[i + j] == b'=' {
                padding += 6;
                continue;
            }
            let x = (|x| {
                if b'A' <= x && x <= b'Z' {
                    Some(x - b'A')
                } else if b'a' <= x && x <= b'z' {
                    Some(x - b'a' + 26)
                } else if b'0' <= x && x <= b'9' {
                    Some(x - b'0' + 52)
                } else if b'-' == x || b'+' == x {
                    Some(62)
                } else if b'_' == x || b'/' == x {
                    Some(63)
                } else {
                    None
                }
            })(data[i + j]);
            if let Some(x) = x {
                group |= (x as u32) << (18 - 6 * j);
            } else {
                if let Some(ignore_error) = ignore_error {
                    if ignore_error == Error::WrongSymbol
                        || ignore_error == Error::PaddingWrongSymbol
                    {
                        break;
                    }
                }
                return Err(Error::WrongSymbol);
            }
            length += 6;
        }
        if (length > 0 && length + padding != 24) || padding > 18 {
            if let Some(ignore_error) = ignore_error {
                if ignore_error != Error::Padding && ignore_error != Error::PaddingWrongSymbol {
                    return Err(Error::Padding);
                }
            } else {
                return Err(Error::Padding);
            }
        }
        for j in 0..3 {
            if length < 8 {
                break;
            }
            result.push(((group >> (16 - j * 8))) as u8 & 0xff);
            length -= 8;
        }
        i += 4;
        if padding > 0 && i < data.len() {
            return Err(Error::Padding);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod test {
    #[test]
    fn encode() {
        let data_one = b"\x01";
        let data_two = b"\x01\x23";
        let data_three = b"\x01\x23\x45";
        let data_full = {
            let mut tmp = vec![0u8; 256];
            for (i, x) in tmp.iter_mut().enumerate() {
                *x = i as u8;
            }
            tmp
        };

        let data_full_assert = b"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gI\
        SIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0+P0BBQkNERUZHSElKS0xNTk9QUVJTVFV\
        WV1hZWltcXV5fYGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn+AgYKDhIWGh4iJi\
        ouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq+wsbKztLW2t7i5uru8vb6\
        /wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t/g4eLj5OXm5+jp6uvs7e7v8PHy8\
        /T19vf4+fr7/P3+/w==";
        let data_full_assert_slice = unsafe {
            ::std::slice::from_raw_parts(data_full_assert.as_ptr(), data_full_assert.len())
        };

        assert_eq!(&super::encode(data_one), &b"AQ==");
        assert_eq!(&super::encode_without_pading(data_one), &b"AQ");
        assert_eq!(&super::encode(data_two), &b"ASM=");
        assert_eq!(&super::encode_without_pading(data_two), &b"ASM");
        assert_eq!(&super::encode(data_three), &b"ASNF");
        assert_eq!(&super::encode(&data_full), &data_full_assert_slice);
        assert_eq!(
            &super::encode_without_pading(&data_full),
            &&data_full_assert_slice[0..data_full_assert.len() - 2]
        );
        assert_eq!(&super::encode(b"\xfb\xef\xff"), &b"++//");
        assert_eq!(&super::urlsafe_encode(b"\xfb\xef\xff"), &b"--__");
    }

    #[test]
    fn decode() {
        let data_one_assert = b"\x01";
        let data_two_assert = b"\x01\x23";
        let data_three_assert = b"\x01\x23\x45";
        let data_full_assert = {
            let mut tmp = vec![0u8; 256];
            for (i, x) in tmp.iter_mut().enumerate() {
                *x = i as u8;
            }
            tmp
        };
        let data_full = b"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gI\
        SIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0+P0BBQkNERUZHSElKS0xNTk9QUVJTVFV\
        WV1hZWltcXV5fYGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn+AgYKDhIWGh4iJi\
        ouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq+wsbKztLW2t7i5uru8vb6\
        /wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t/g4eLj5OXm5+jp6uvs7e7v8PHy8\
        /T19vf4+fr7/P3+/w==";
        let data_full_slice =
            unsafe { ::std::slice::from_raw_parts(data_full.as_ptr(), data_full.len()) };

        use super::Error as DE;
        assert_eq!(&super::decode(b"AQ==", None).unwrap(), &data_one_assert);
        assert_eq!(&super::decode(b"ASM=", None).unwrap(), &data_two_assert);
        assert_eq!(&super::decode(b"ASNF", None).unwrap(), &data_three_assert);
        assert_eq!(
            &super::decode(data_full_slice, None).unwrap(),
            &data_full_assert
        );
        assert_eq!(
            &super::decode(b"AQ", Some(DE::Padding)).unwrap(),
            &data_one_assert
        );
        assert_eq!(
            &super::decode(b"ASM", Some(DE::Padding)).unwrap(),
            &data_two_assert
        );
        assert_eq!(&super::decode(b"ASM", None).unwrap_err(), &DE::Padding);
        assert_eq!(&super::decode(b"ASM ", None).unwrap_err(), &DE::WrongSymbol);
        assert_eq!(&super::decode(b"++//", None).unwrap(), b"\xfb\xef\xff");
        assert_eq!(&super::decode(b"--__", None).unwrap(), b"\xfb\xef\xff");
    }
}
