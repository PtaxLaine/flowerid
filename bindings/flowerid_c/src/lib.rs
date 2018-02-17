extern crate flowerid;
extern crate libc;
use self::libc::*;
use std::ptr;
use std::mem;
use std::slice;

use flowerid::id;
use flowerid::config;
use flowerid::generator;
use flowerid::Error;

#[allow(non_camel_case_types)]
pub type FID_C = uint64_t;
#[allow(non_camel_case_types)]
pub type FIDGEN_C = *mut generator::FIDGenerator;

#[derive(Debug, PartialEq, PartialOrd)]
enum RESULT {
    OK = 0,
    InvalidArgument = -1,
    TimestampOverflow = -2,
    SequenceOverflow = -3,
    GeneratorOverflow = -4,
    SysTimeIsInPast = -5,
    WrongSliceSize = -6,
    Base64DecodeError = -7,
    BufferWrongSize = -8,
}

impl From<RESULT> for i32 {
    fn from(res: RESULT) -> i32 {
        res as i32
    }
}

impl From<RESULT> for i64 {
    fn from(res: RESULT) -> i64 {
        res as i64
    }
}

fn from_errori32(res: Error) -> i32 {
    use Error::*;
    match res {
        TimestampOverflow(_) => From::from(RESULT::TimestampOverflow),
        SequenceOverflow(_) => From::from(RESULT::SequenceOverflow),
        GeneratorOverflow(_) => From::from(RESULT::GeneratorOverflow),
        SysTimeIsInPast => From::from(RESULT::SysTimeIsInPast),
        WrongSliceSize(_) => From::from(RESULT::WrongSliceSize),
        Base64PaddingError => From::from(RESULT::Base64DecodeError),
        Base64BufferTooSmall => From::from(RESULT::Base64DecodeError),
        Base64WrongSymbolError => From::from(RESULT::Base64DecodeError),
        Base64PaddingWrongSymbolError => From::from(RESULT::Base64DecodeError),
    }
}

unsafe fn flowerid_rust_to_c(dst: *mut FID_C, src: &id::FID) {
    *dst = From::from(src.clone());
}

fn flowerid_c_to_rust(src: FID_C) -> id::FID {
    From::from(src)
}

#[no_mangle]
pub extern "C" fn flowerid_new(
    dst: *mut FID_C,
    timestamp: uint64_t,
    sequence: uint64_t,
    generator: uint64_t,
) -> int32_t {
    if dst == ptr::null_mut() {
        return From::from(RESULT::InvalidArgument);
    }
    match id::FID::new(timestamp, sequence as u16, generator as u16) {
        Ok(id) => unsafe {
            flowerid_rust_to_c(dst, &id);
            From::from(RESULT::OK)
        },
        Err(x) => from_errori32(x),
    }
}

#[no_mangle]
pub extern "C" fn flowerid_to_bytes(
    this: FID_C,
    buffer: *mut uint8_t,
    buffer_size: size_t,
) -> int32_t {
    if buffer == ptr::null_mut() {
        return From::from(RESULT::InvalidArgument);
    }
    if buffer_size != 8 {
        return From::from(RESULT::BufferWrongSize);
    }
    let result = id::FID::from(this).to_bytes();
    for (i, x) in result.iter().enumerate() {
        unsafe {
            *buffer.offset(i as isize) = *x;
        }
    }
    result.len() as i32
}

#[no_mangle]
pub extern "C" fn flowerid_from_bytes(
    dst: *mut FID_C,
    buffer: *const uint8_t,
    buffer_size: size_t,
) -> int32_t {
    if dst == ptr::null_mut() || buffer == ptr::null() {
        return From::from(RESULT::InvalidArgument);
    }
    if buffer_size != 8 {
        return From::from(RESULT::BufferWrongSize);
    }
    unsafe {
        let mut tmp: [u8; 8] = mem::uninitialized();
        for (i, x) in tmp.iter_mut().enumerate() {
            *x = *buffer.offset(i as isize);
        }
        let id = id::FID::from_bytes(&tmp);
        flowerid_rust_to_c(dst, &id);
        From::from(RESULT::OK)
    }
}

#[no_mangle]
pub extern "C" fn flowerid_to_string(
    this: FID_C,
    buffer: *mut c_char,
    buffer_size: size_t,
) -> int32_t {
    if buffer == ptr::null_mut() {
        return From::from(RESULT::InvalidArgument);
    }
    if buffer_size != 12 {
        return From::from(RESULT::BufferWrongSize);
    }
    let result = id::FID::from(this).to_b64();
    unsafe {
        for (i, x) in result.iter().enumerate() {
            *buffer.offset(i as isize) = *x as c_char;
        }
        *buffer.offset(11) = 0;
    }
    11
}

#[no_mangle]
pub extern "C" fn flowerid_from_string(dst: *mut FID_C, buffer: *const c_char) -> int32_t {
    if dst == ptr::null_mut() || buffer == ptr::null() {
        return From::from(RESULT::InvalidArgument);
    }
    if unsafe { strlen(buffer) } != 11 {
        return From::from(RESULT::BufferWrongSize);
    }
    let buffer = unsafe { slice::from_raw_parts(buffer as *const u8, 11) };
    match id::FID::from_b64(buffer) {
        Ok(id) => unsafe {
            flowerid_rust_to_c(dst, &id);
            From::from(RESULT::OK)
        },
        Err(err) => from_errori32(err),
    }
}

#[no_mangle]
pub extern "C" fn flowerid_get_timestamp(dst: FID_C) -> uint64_t {
    let id = flowerid_c_to_rust(dst);
    id.timestamp()
}

#[no_mangle]
pub extern "C" fn flowerid_get_sequence(dst: FID_C) -> uint64_t {
    let id = flowerid_c_to_rust(dst);
    id.sequence() as u64
}

#[no_mangle]
pub extern "C" fn flowerid_get_generator(dst: FID_C) -> uint64_t {
    let id = flowerid_c_to_rust(dst);
    id.generator() as u64
}

#[no_mangle]
pub extern "C" fn flowerid_generator_new(
    dst: *mut FIDGEN_C,
    generator: uint64_t,
    wait_sequence: uint32_t,
) -> int32_t {
    flowerid_generator_new_ex(
        dst,
        generator,
        config::gbuilder_defaults::TIMESTAMP_OFFSET,
        0,
        0,
        wait_sequence,
        if config::gbuilder_defaults::TIMESTAMP_IN_SECONDS {
            1
        } else {
            0
        },
    )
}

#[no_mangle]
pub extern "C" fn flowerid_generator_new_ex(
    dst: *mut FIDGEN_C,
    generator: uint64_t,
    timestamp_offset: int64_t,
    timestamp_last: uint64_t,
    sequence: uint64_t,
    wait_sequence: uint32_t,
    timestamp_in_seconds: uint32_t,
) -> int32_t {
    if dst == ptr::null_mut() {
        return From::from(RESULT::InvalidArgument);
    }
    let mut gen = generator::FIDGeneratorBuilder::new(generator as u16)
        .timestamp_offset(timestamp_offset)
        .timestamp_last(timestamp_last)
        .sequence(sequence as u16)
        .wait_sequence(!(wait_sequence == 0));
    if timestamp_in_seconds == 0 {
        gen = gen.timestamp_in_milliseconds();
    } else {
        gen = gen.timestamp_in_seconds();
    }
    match generator::FIDGenerator::new(gen) {
        Ok(gen) => unsafe {
            *dst = Box::into_raw(Box::new(gen));
            From::from(RESULT::OK)
        },
        Err(err) => from_errori32(err),
    }
}

#[no_mangle]
pub extern "C" fn flowerid_generator_next(this: FIDGEN_C, dst: *mut FID_C) -> int32_t {
    if this == ptr::null_mut() || dst == ptr::null_mut() {
        return From::from(RESULT::InvalidArgument);
    }
    unsafe {
        match (*this).next() {
            Ok(id) => {
                flowerid_rust_to_c(dst, &id);
                From::from(RESULT::OK)
            }
            Err(err) => from_errori32(err),
        }
    }
}

#[no_mangle]
pub extern "C" fn flowerid_generator_release(this: FIDGEN_C) -> int32_t {
    if this == ptr::null_mut() {
        return From::from(RESULT::InvalidArgument);
    }
    unsafe {
        drop(Box::from_raw(this));
    }
    From::from(RESULT::OK)
}
