//! Flower identificator generator
use std::cmp;
use std::time;
use std::thread;
use id::FID;
use config as cfg;
#[cfg(not(test))]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(test)]
use stubs::systemtime::{SystemTime, UNIX_EPOCH};

use {Error, Result};

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
    ///         .timestamp_in_milliseconds()
    /// ).unwrap();
    /// ```
    pub fn new(generator: u16) -> FIDGeneratorBuilder {
        FIDGeneratorBuilder(FIDGenerator {
            generator: generator,
            timestamp_offset: cfg::gbuilder_defaults::TIMESTAMP_OFFSET,
            timestamp_last: 0,
            sequence: 0,
            wait_sequence: cfg::gbuilder_defaults::WAIT_SEQUENCE,
            timestamp_in_seconds: cfg::gbuilder_defaults::TIMESTAMP_IN_SECONDS,
        })
    }

    /// Set timestamp last timestamp
    pub fn timestamp_last(mut self, val: u64) -> FIDGeneratorBuilder {
        self.0.timestamp_last = val;
        self
    }

    /// Set timestamp in seconds
    pub fn timestamp_in_seconds(mut self) -> FIDGeneratorBuilder {
        self.0.timestamp_in_seconds = true;
        self
    }

    /// Set timestamp in seconds
    pub fn timestamp_in_milliseconds(mut self) -> FIDGeneratorBuilder {
        self.0.timestamp_in_seconds = false;
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
    ///
    /// If wait_sequence is true and sequence overflowed, generator wait till next timetamp has been generated
    pub fn wait_sequence(mut self, val: bool) -> FIDGeneratorBuilder {
        self.0.wait_sequence = val;
        self
    }

    /// Build `FIDGenerator`
    ///
    /// alike `FIDGenerator::new(self)`
    pub fn build(self) -> Result<FIDGenerator> {
        FIDGenerator::new(self)
    }
}

impl FIDGenerator {
    /// Create new generator
    ///
    /// # Failures
    /// `Error::GeneratorOverflow`
    /// `Error::SequenceOverflow`
    /// `Error::TimestampOverflow`
    pub fn new(cfg: FIDGeneratorBuilder) -> Result<FIDGenerator> {
        if cfg.0.generator >= 1 << cfg::GENERATOR_LENGTH {
            Err(Error::GeneratorOverflow(cfg.0.generator))
        } else if cfg.0.sequence >= 1 << cfg::SEQUENCE_LENGTH {
            Err(Error::SequenceOverflow(cfg.0.sequence))
        } else if cfg.0.timestamp_last >= 1 << cfg::TIMESTAMP_LENGTH {
            Err(Error::TimestampOverflow(cfg.0.timestamp_last))
        } else {
            Ok(cfg.0)
        }
    }

    /// Generate next id
    ///
    /// # Failures
    /// `Error::SequenceOverflow`
    /// `Error::SysTimeIsInPast`
    /// `Error::TimestampOverflow`
    ///
    /// # Examples
    /// ```
    /// use flowerid::generator::*;
    /// let mut gen = FIDGenerator::new(FIDGeneratorBuilder::new(0x12c)).unwrap();
    /// println!("{}", gen.next().unwrap());
    /// ```
    pub fn next(&mut self) -> Result<FID> {
        let timestamp = self.new_timestamp()?;

        match timestamp.cmp(&self.timestamp_last) {
            cmp::Ordering::Less => Err(Error::SysTimeIsInPast),
            cmp::Ordering::Greater => self.next_timestamp(timestamp),
            cmp::Ordering::Equal => self.next_sequence(timestamp),
        }
    }

    fn new_timestamp(&self) -> Result<u64> {
        let convert_time = |time: &time::Duration| -> u64 {
            if self.timestamp_in_seconds {
                time.as_secs()
            } else {
                time.as_secs() * 1000 + (time.subsec_nanos() / 1000 / 1000) as u64
            }
        };

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
        let timestamp = convert_time(&timestamp);
        if timestamp >= (1 << cfg::TIMESTAMP_LENGTH) {
            Err(Error::TimestampOverflow(timestamp))
        } else {
            Ok(timestamp)
        }
    }

    fn next_timestamp(&mut self, timestamp: u64) -> Result<FID> {
        self.timestamp_last = timestamp;
        self.sequence = 0;
        FID::new(timestamp, 0, self.generator)
    }

    fn wait_next_timestamp(&self) -> Result<()> {
        let start_time = SystemTime::now();
        loop {
            if let Ok(duration_since) = SystemTime::now().duration_since(start_time) {
                if self.timestamp_in_seconds {
                    if duration_since.as_secs() > 0 {
                        return Ok(());
                    } else {
                        thread::sleep(time::Duration::from_millis(10));
                    }
                } else {
                    if duration_since.subsec_nanos() / 1000 / 1000 > 0 {
                        return Ok(());
                    } else {
                        thread::sleep(time::Duration::from_millis(1));
                    }
                }
            } else {
                return Err(Error::SysTimeIsInPast);
            }
        }
    }

    fn next_sequence(&mut self, timestamp: u64) -> Result<FID> {
        if (self.sequence + 1) >= (1 << cfg::SEQUENCE_LENGTH) {
            if self.wait_sequence {
                self.wait_next_timestamp()?;
                return self.next();
            } else {
                return Err(Error::SequenceOverflow(self.sequence));
            }
        } else {
            self.sequence += 1;
            return FID::new(timestamp, self.sequence, self.generator);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use config::test_constants::*;

    #[test]
    fn builder() {
        let gen = FIDGeneratorBuilder::new(GENERATOR);
        assert_eq!(gen.0.generator, GENERATOR);
        assert_eq!(gen.0.sequence, 0);
        assert_eq!(gen.0.timestamp_last, 0);
        assert_eq!(
            gen.0.timestamp_offset,
            cfg::gbuilder_defaults::TIMESTAMP_OFFSET
        );
        assert_eq!(gen.0.wait_sequence, cfg::gbuilder_defaults::WAIT_SEQUENCE);
        assert_eq!(
            gen.0.timestamp_in_seconds,
            cfg::gbuilder_defaults::TIMESTAMP_IN_SECONDS
        );
        let mut gen = FIDGeneratorBuilder::new(0)
            .sequence(GENERATOR / 2)
            .timestamp_last(TIMESTAMP / 2)
            .timestamp_offset(-1800)
            .wait_sequence(!cfg::gbuilder_defaults::WAIT_SEQUENCE);
        if cfg::gbuilder_defaults::TIMESTAMP_IN_SECONDS {
            gen = gen.timestamp_in_milliseconds();
        } else {
            gen = gen.timestamp_in_seconds();
        }
        assert_eq!(gen.0.generator, 0);
        assert_eq!(gen.0.sequence, GENERATOR / 2);
        assert_eq!(gen.0.timestamp_last, TIMESTAMP / 2);
        assert_eq!(gen.0.timestamp_offset, -1800);
        assert_eq!(gen.0.wait_sequence, !cfg::gbuilder_defaults::WAIT_SEQUENCE);
        assert_eq!(
            gen.0.timestamp_in_seconds,
            !cfg::gbuilder_defaults::TIMESTAMP_IN_SECONDS
        );
    }

    #[test]
    fn new() {
        FIDGenerator::new(FIDGeneratorBuilder::new(GENERATOR)).unwrap();
        assert_eq!(
            FIDGenerator::new(FIDGeneratorBuilder::new(1 << cfg::GENERATOR_LENGTH)).unwrap_err(),
            Error::GeneratorOverflow(1 << cfg::GENERATOR_LENGTH)
        );
        assert_eq!(
            FIDGenerator::new(
                FIDGeneratorBuilder::new(GENERATOR).sequence(1 << cfg::SEQUENCE_LENGTH)
            ).unwrap_err(),
            Error::SequenceOverflow(1 << cfg::SEQUENCE_LENGTH)
        );
        assert_eq!(
            FIDGenerator::new(
                FIDGeneratorBuilder::new(GENERATOR).timestamp_last(1 << cfg::TIMESTAMP_LENGTH)
            ).unwrap_err(),
            Error::TimestampOverflow(1 << cfg::TIMESTAMP_LENGTH)
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn SysTimeIsInPast() {
        let mut lock_sys_time =
            SystemTime::lock((cfg::gbuilder_defaults::TIMESTAMP_OFFSET.abs() - 1) * 1000);
        let mut gen =
            FIDGenerator::new(FIDGeneratorBuilder::new(GENERATOR).wait_sequence(false)).unwrap();
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
        let lock_sys_time = SystemTime::lock(
            cfg::gbuilder_defaults::TIMESTAMP_OFFSET.abs() * 1000 + (1 << cfg::TIMESTAMP_LENGTH),
        );
        let mut gen =
            FIDGenerator::new(FIDGeneratorBuilder::new(GENERATOR).wait_sequence(false)).unwrap();
        assert_eq!(
            gen.next().unwrap_err(),
            Error::TimestampOverflow(1 << cfg::TIMESTAMP_LENGTH)
        );
        SystemTime::unlock(lock_sys_time);
    }

    #[test]
    #[allow(non_snake_case)]
    fn SequenceOverflow() {
        let lock_sys_time = SystemTime::lock(
            cfg::gbuilder_defaults::TIMESTAMP_OFFSET.abs() * 1000 + TIMESTAMP as i64,
        );
        let mut gen = FIDGenerator::new(
            FIDGeneratorBuilder::new(GENERATOR)
                .wait_sequence(false)
                .timestamp_in_seconds(),
        ).unwrap();
        for i in 0..(1 << cfg::SEQUENCE_LENGTH) {
            let fid = gen.next().unwrap();
            assert_eq!(fid.timestamp(), TIMESTAMP / 1000);
            assert_eq!(fid.generator(), GENERATOR);
            assert_eq!(fid.sequence(), i);
        }
        assert_eq!(
            gen.next().unwrap_err(),
            Error::SequenceOverflow((1 << cfg::SEQUENCE_LENGTH) - 1)
        );
        SystemTime::unlock(lock_sys_time);

        let mut lock_sys_time = SystemTime::lock(
            cfg::gbuilder_defaults::TIMESTAMP_OFFSET.abs() * 1000 + TIMESTAMP as i64,
        );
        let mut gen =
            FIDGenerator::new(FIDGeneratorBuilder::new(GENERATOR).wait_sequence(false)).unwrap();
        for i in 0..(1 << cfg::SEQUENCE_LENGTH) {
            let fid = gen.next().unwrap();
            assert_eq!(fid.timestamp(), TIMESTAMP);
            assert_eq!(fid.generator(), GENERATOR);
            assert_eq!(fid.sequence(), i);
        }
        assert_eq!(
            gen.next().unwrap_err(),
            Error::SequenceOverflow((1 << cfg::SEQUENCE_LENGTH) - 1)
        );
        lock_sys_time.add(1);
        for i in 0..(1 << cfg::SEQUENCE_LENGTH) {
            let fid = gen.next().unwrap();
            assert_eq!(fid.timestamp(), TIMESTAMP + 1);
            assert_eq!(fid.generator(), GENERATOR);
            assert_eq!(fid.sequence(), i);
        }
        assert_eq!(
            gen.next().unwrap_err(),
            Error::SequenceOverflow((1 << cfg::SEQUENCE_LENGTH) - 1)
        );
        SystemTime::unlock(lock_sys_time);
    }

    #[test]
    fn next() {
        let lock_sys_time = SystemTime::lock(
            cfg::gbuilder_defaults::TIMESTAMP_OFFSET.abs() * 1000 + TIMESTAMP as i64,
        );
        let mut gen =
            FIDGenerator::new(FIDGeneratorBuilder::new(GENERATOR).wait_sequence(false)).unwrap();
        for i in 0..(1 << cfg::SEQUENCE_LENGTH) - 1 {
            let fid = gen.next().unwrap();
            assert_eq!(fid.timestamp(), TIMESTAMP);
            assert_eq!(fid.sequence(), i);
            assert_eq!(fid.generator(), GENERATOR);
        }
        SystemTime::unlock(lock_sys_time);
    }
}
