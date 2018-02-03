use std::ops;
use std::time;
use std::sync::atomic;

pub const UNIX_EPOCH: SystemTime = SystemTime(0);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SystemTime(i64);

#[derive(Debug)]
pub struct SystemTimeLock;

#[derive(Debug, Clone, Copy)]
struct SystemTimeInstance {
    start_value: i64,
    locked: bool,
}

impl SystemTime {
    pub fn now() -> SystemTime {
        assert!(SystemTime::instance().locked);
        SystemTime(SystemTime::instance().start_value)
    }

    pub fn duration_since(&self, earlier: SystemTime) -> Result<time::Duration, ()> {
        if self.0 < earlier.0 {
            unreachable!()
        } else {
            Ok(time::Duration::from_millis((self.0 - earlier.0) as u64))
        }
    }

    fn instance() -> &'static mut SystemTimeInstance {
        static mut INST: SystemTimeInstance = SystemTimeInstance {
            start_value: 0,
            locked: false,
        };
        unsafe { &mut INST }
    }

    fn flag_instance() -> &'static mut atomic::AtomicBool {
        static mut INST: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;
        unsafe { &mut INST }
    }

    pub fn lock(start_value: i64) -> SystemTimeLock {
        loop {
            if SystemTime::flag_instance().swap(true, atomic::Ordering::Acquire) == false {
                break;
            }
        }
        *SystemTime::instance() = SystemTimeInstance {
            start_value,
            locked: true,
        };
        SystemTimeLock
    }

    pub fn unlock(locker: SystemTimeLock) {
        drop(locker);
    }
}

impl SystemTimeLock {
    pub fn add(&mut self, value: i64) {
        SystemTime::instance().start_value += value;
    }
}

impl Drop for SystemTimeLock {
    fn drop(&mut self) {
        SystemTime::instance().locked = false;
        SystemTime::flag_instance().store(false, atomic::Ordering::Release);
    }
}

impl ops::AddAssign<time::Duration> for SystemTime {
    fn add_assign(&mut self, other: time::Duration) {
        self.0 += (other.as_secs() * 1000 + ((other.subsec_nanos() / 1000 / 1000) as u64)) as i64;
    }
}

impl ops::SubAssign<time::Duration> for SystemTime {
    fn sub_assign(&mut self, other: time::Duration) {
        self.0 -= (other.as_secs() * 1000 + ((other.subsec_nanos() / 1000 / 1000) as u64)) as i64;
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn basic() {
        use std::time::Duration;
        use super::SystemTime;

        assert_eq!(super::UNIX_EPOCH, SystemTime(0));

        let mut lock = SystemTime::lock(4096);
        let diff = SystemTime::now().duration_since(SystemTime(2048)).unwrap();
        assert_eq!(diff.as_secs(), 2);
        assert_eq!(diff.subsec_nanos(), 48 * 1000 * 1000);

        let mut time = SystemTime(1024);
        time += Duration::from_millis(2048);
        let diff = SystemTime::now().duration_since(time).unwrap();
        assert_eq!(diff.as_secs(), 1);
        assert_eq!(diff.subsec_nanos(), 24 * 1000 * 1000);

        let mut time = SystemTime(2048);
        time -= Duration::from_millis(1004);
        let diff = SystemTime::now().duration_since(time).unwrap();
        assert_eq!(diff.as_secs(), 3);
        assert_eq!(diff.subsec_nanos(), 52 * 1000 * 1000);

        lock.add(1234);
        let diff = SystemTime::now().duration_since(SystemTime(0)).unwrap();
        assert_eq!(diff.as_secs(), 5);
        assert_eq!(diff.subsec_nanos(), 330 * 1000 * 1000);
    }
}
