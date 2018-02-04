//! Flower identificator generator
use std::cmp;
use std::time;
use std::thread;
use id::FID;
use limits as lim;
#[cfg(not(test))]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(test)]
use stub_time_systemtime::{SystemTime, UNIX_EPOCH};

use {Error, Result};

const TIMESTAMP_OFFSET: i64 = -1483228800;

/// Flower identificator generator
#[derive(Debug, Clone)]
pub struct FIDGenerator {
    generator: u16,
    timestamp_offset: i64,
    timestamp_last: u64,
    sequence: u16,
    wait_sequence: bool,
    timestamp_in_seconds: bool,
}

/// Generator configuration builder
#[derive(Debug, Clone)]
pub struct FIDGeneratorBuilder(FIDGenerator);

impl FIDGeneratorBuilder {
    /// Create builder
    ///
    /// # Examples
    /// ```
    /// use flowerid::generator::*;
    /// let gen = FIDGenerator::new(
    ///     FIDGeneratorBuilder::new(0)
    ///         .timestamp_last(0)
    ///         .sequence(0)
    ///         .timestamp_offset(-1483228800)
    ///         .wait_sequence(true)
    ///         .timestamp_in_seconds(false)
    /// ).unwrap();
    /// ```
    pub fn new(generator: u16) -> FIDGeneratorBuilder {
        FIDGeneratorBuilder(FIDGenerator {
            generator: generator,
            timestamp_offset: TIMESTAMP_OFFSET,
            timestamp_last: 0,
            sequence: 0,
            wait_sequence: true,
            timestamp_in_seconds: false,
        })
    }

    /// Set timestamp last timestamp
    pub fn timestamp_last(mut self, val: u64) -> FIDGeneratorBuilder {
        self.0.timestamp_last = val;
        self
    }

    /// Set timestamp in seconds
    pub fn timestamp_in_seconds(mut self, val: bool) -> FIDGeneratorBuilder {
        self.0.timestamp_in_seconds = val;
        self
    }

    /// Set sequence
    pub fn sequence(mut self, val: u16) -> FIDGeneratorBuilder {
        self.0.sequence = val;
        self
    }

    /// Set timestamp offset
    pub fn timestamp_offset(mut self, val: i64) -> FIDGeneratorBuilder {
        self.0.timestamp_offset = val;
        self
    }

    /// Set wait sequence
    /// If wait_sequence is true and sequence overflowed, generator wait till next timetamp has been generated
    pub fn wait_sequence(mut self, val: bool) -> FIDGeneratorBuilder {
        self.0.wait_sequence = val;
        self
    }
}

impl FIDGenerator {
    /// Create new generator
    ///
    /// # Failures
    /// Error::GeneratorOverflow
    /// Error::SequenceOverflow
    /// Error::TimestampOverflow
    pub fn new(cfg: FIDGeneratorBuilder) -> Result<FIDGenerator> {
        if cfg.0.generator >= 1 << lim::GENERATOR_LENGTH {
            Err(Error::GeneratorOverflow(cfg.0.generator))
        } else if cfg.0.sequence >= 1 << lim::SEQUENCE_LENGTH {
            Err(Error::SequenceOverflow(cfg.0.sequence))
        } else if cfg.0.timestamp_last >= 1 << lim::TIMESTAMP_LENGTH {
            Err(Error::TimestampOverflow(cfg.0.timestamp_last))
        } else {
            Ok(cfg.0)
        }
    }

    /// Generate next id
    ///
    /// # Failures
    /// Error::SequenceOverflow
    /// Error::SysTimeIsInPast
    /// Error::TimestampOverflow
    ///
    /// # Examples
    /// ```
    /// use flowerid::generator::*;
    /// let mut gen = FIDGenerator::new(FIDGeneratorBuilder::new(0x12c)).unwrap();
    /// println!("{}", gen.next().unwrap());
    /// ```
    pub fn next(&mut self) -> Result<FID> {
        fn convert_time(in_sec: bool, time: &time::Duration) -> u64 {
            if in_sec {
                time.as_secs()
            } else {
                time.as_secs() * 1000 + (time.subsec_nanos() / 1000 / 1000) as u64
            }
        }

        let mut offset = UNIX_EPOCH;
        if self.timestamp_offset < 0 {
            offset += time::Duration::from_secs(self.timestamp_offset.abs() as u64);
        } else {
            offset -= time::Duration::from_secs(self.timestamp_offset.abs() as u64);
        }
        let sys_time = SystemTime::now();
        if sys_time < offset {
            return Err(Error::SysTimeIsInPast);
        }
        let timestamp = sys_time
            .duration_since(offset)
            .map_err(|_| Error::SysTimeIsInPast)?;
        let timestamp = convert_time(self.timestamp_in_seconds, &timestamp);
        if timestamp >= 1 << lim::TIMESTAMP_LENGTH {
            return Err(Error::TimestampOverflow(timestamp));
        }
        match timestamp.cmp(&self.timestamp_last) {
            cmp::Ordering::Less => Err(Error::SysTimeIsInPast),
            cmp::Ordering::Greater => {
                self.timestamp_last = timestamp;
                self.sequence = 0;
                Ok(FID::new(timestamp, 0, self.generator).unwrap())
            }
            cmp::Ordering::Equal => {
                if (self.sequence + 1) >= 1 << lim::SEQUENCE_LENGTH {
                    if self.wait_sequence {
                        loop {
                            if let Ok(duration_since) = SystemTime::now().duration_since(sys_time) {
                                if self.timestamp_in_seconds {
                                    if duration_since.as_secs() > 0 {
                                        break;
                                    } else {
                                        thread::sleep(time::Duration::from_millis(10));
                                    }
                                } else {
                                    if duration_since.subsec_nanos() / 1000 / 1000 > 0 {
                                        break;
                                    } else {
                                        thread::sleep(time::Duration::from_millis(1));
                                    }
                                }
                            } else {
                                return Err(Error::SysTimeIsInPast);
                            }
                        }
                        return self.next();
                    }
                    return Err(Error::SequenceOverflow(self.sequence));
                }
                self.sequence += 1;
                Ok(FID::new(timestamp, self.sequence, self.generator).unwrap())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn builder() {
        let gen = FIDGeneratorBuilder::new(0x272);
        assert_eq!(gen.0.generator, 0x272);
        assert_eq!(gen.0.sequence, 0);
        assert_eq!(gen.0.timestamp_last, 0);
        assert_eq!(gen.0.timestamp_offset, TIMESTAMP_OFFSET);
        assert_eq!(gen.0.wait_sequence, true);
        let gen = FIDGeneratorBuilder::new(0)
            .sequence(0x436)
            .timestamp_last(45462)
            .timestamp_offset(-1800)
            .wait_sequence(false);
        assert_eq!(gen.0.generator, 0);
        assert_eq!(gen.0.sequence, 0x436);
        assert_eq!(gen.0.timestamp_last, 45462);
        assert_eq!(gen.0.timestamp_offset, -1800);
        assert_eq!(gen.0.wait_sequence, false);
    }

    #[test]
    fn new() {
        FIDGenerator::new(FIDGeneratorBuilder::new(0x249)).unwrap();
        assert_eq!(
            FIDGenerator::new(FIDGeneratorBuilder::new(1 << 10)).unwrap_err(),
            Error::GeneratorOverflow(1 << 10)
        );
        assert_eq!(
            FIDGenerator::new(FIDGeneratorBuilder::new(0x249).sequence(1 << 11)).unwrap_err(),
            Error::SequenceOverflow(1 << 11)
        );
        assert_eq!(
            FIDGenerator::new(FIDGeneratorBuilder::new(0x249).timestamp_last(1 << 42)).unwrap_err(),
            Error::TimestampOverflow(1 << 42)
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn SysTimeIsInPast() {
        let mut lock_sys_time = SystemTime::lock(1483228799 * 1000);
        let mut gen =
            FIDGenerator::new(FIDGeneratorBuilder::new(0x249).wait_sequence(false)).unwrap();
        assert_eq!(gen.next().unwrap_err(), Error::SysTimeIsInPast);
        lock_sys_time.add(1001);
        assert_eq!(gen.next().unwrap().timestamp(), 1);
        lock_sys_time.add(3);
        assert_eq!(gen.next().unwrap().timestamp(), 4);
        lock_sys_time.add(-1);
        assert_eq!(gen.next().unwrap_err(), Error::SysTimeIsInPast);
    }

    #[test]
    #[allow(non_snake_case)]
    fn TimestampOverflow() {
        let lock_sys_time = SystemTime::lock(TIMESTAMP_OFFSET.abs() * 1000 + 4398046511104);
        let mut gen =
            FIDGenerator::new(FIDGeneratorBuilder::new(0x249).wait_sequence(false)).unwrap();
        assert_eq!(
            gen.next().unwrap_err(),
            Error::TimestampOverflow(4398046511104)
        );
        SystemTime::unlock(lock_sys_time);
    }

    #[test]
    #[allow(non_snake_case)]
    fn SequenceOverflow() {
        let lock_sys_time = SystemTime::lock(TIMESTAMP_OFFSET.abs() * 1000 + 2073867450856);
        let mut gen = FIDGenerator::new(
            FIDGeneratorBuilder::new(0x249)
                .wait_sequence(false)
                .timestamp_in_seconds(true),
        ).unwrap();
        for i in 0..2048 {
            let fid = gen.next().unwrap();
            assert_eq!(fid.timestamp(), 2073867450);
            assert_eq!(fid.generator(), 0x249);
            assert_eq!(fid.sequence(), i);
        }
        assert_eq!(gen.next().unwrap_err(), Error::SequenceOverflow(2047));
        SystemTime::unlock(lock_sys_time);

        let mut lock_sys_time = SystemTime::lock(TIMESTAMP_OFFSET.abs() * 1000 + 2073867450856);
        let mut gen =
            FIDGenerator::new(FIDGeneratorBuilder::new(0x249).wait_sequence(false)).unwrap();
        for i in 0..2048 {
            let fid = gen.next().unwrap();
            assert_eq!(fid.timestamp(), 2073867450856);
            assert_eq!(fid.generator(), 0x249);
            assert_eq!(fid.sequence(), i);
        }
        assert_eq!(gen.next().unwrap_err(), Error::SequenceOverflow(2047));
        lock_sys_time.add(1);
        for i in 0..2048 {
            let fid = gen.next().unwrap();
            assert_eq!(fid.timestamp(), 2073867450856 + 1);
            assert_eq!(fid.generator(), 0x249);
            assert_eq!(fid.sequence(), i);
        }
        assert_eq!(gen.next().unwrap_err(), Error::SequenceOverflow(2047));
        SystemTime::unlock(lock_sys_time);
    }

    #[test]
    fn next() {
        let lock_sys_time = SystemTime::lock(TIMESTAMP_OFFSET.abs() * 1000);
        let mut gen =
            FIDGenerator::new(FIDGeneratorBuilder::new(0x249).wait_sequence(false)).unwrap();
        assert_eq!(gen.next().unwrap().timestamp(), 0);
        SystemTime::unlock(lock_sys_time);
    }
}
