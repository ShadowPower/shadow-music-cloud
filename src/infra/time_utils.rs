use std::time::{SystemTime, UNIX_EPOCH};

pub fn time_to_millis(time: &SystemTime) -> u128 {
    time.duration_since(UNIX_EPOCH).unwrap().as_millis()
}