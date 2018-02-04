extern crate flowerid;

use std::time;
use std::thread;

#[test]
fn usage() {
    fn custom_offset() {
        use flowerid::generator::*;
        let offset = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap();
        let mut gen = FIDGenerator::new(
            FIDGeneratorBuilder::new(0x47)
                .timestamp_offset(-(offset.as_secs() as i64))
                .timestamp_in_seconds(),
        ).unwrap();
        for _ in 0..100 {
            let fid = gen.next().unwrap();
            thread::sleep(time::Duration::from_millis(1));
            println!(
                "{:?} / i: {} / ih: 0x{1:016x}",
                fid.clone(),
                Into::<u64>::into(fid.clone())
            );
        }
    }
    fn default_offset() {
        use flowerid::generator::*;
        let mut gen = FIDGenerator::new(FIDGeneratorBuilder::new(0x47)).unwrap();
        for _ in 0..25 {
            thread::sleep(time::Duration::from_millis(1));
            for _ in 0..4 {
                let fid = gen.next().unwrap();
                println!(
                    "{:?} / i: {} / ih: 0x{1:016x}",
                    fid.clone(),
                    Into::<u64>::into(fid.clone())
                );
            }
        }
    }
    custom_offset();
    default_offset();
}
