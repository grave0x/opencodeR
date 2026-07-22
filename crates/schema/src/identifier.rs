use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const LENGTH: usize = 26;

// Leetopt: direct hex lookup — zero-alloc, no format machinery
const HEX: &[u8; 16] = b"0123456789abcdef";

static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(0);
static COUNTER: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
fn now_ms() -> u64 {
    // SAFETY: SystemTime::duration_since only fails if the clock went backwards,
    // which is a hardware fault — panic is appropriate.
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_millis() as u64
}

pub fn ascending() -> String {
    create(false, None)
}

pub fn descending() -> String {
    create(true, None)
}

pub fn from_timestamp(timestamp: u64, desc: bool) -> String {
    create(desc, Some(timestamp))
}

fn create(desc: bool, timestamp: Option<u64>) -> String {
    let ts = timestamp.unwrap_or_else(now_ms);

    let prev_ts = LAST_TIMESTAMP.swap(ts, Ordering::Relaxed);
    if prev_ts != ts {
        COUNTER.store(0, Ordering::Relaxed);
    }
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

    let current = (ts as u128) * 0x1000u128 + counter as u128;
    let value = if desc { !current } else { current };

    // Stack buffer — zero heap until final allocation
    let mut buf = [0u8; LENGTH];

    // Leetopt: direct hex byte writes via lookup — replaces format!("{:02x}")
    // Each byte → 2 hex chars, no alloc, no fmt dispatch
    for i in 0..6 {
        let shift = 40 - 8 * i;
        let byte = ((value >> shift) & 0xff) as u8;
        buf[i * 2] = HEX[(byte >> 4) as usize];
        buf[i * 2 + 1] = HEX[(byte & 0x0f) as usize];
    }

    // Random suffix via getrandom
    let _ = getrandom::fill(&mut buf[12..]);

    // Leetopt: clamp random bytes to alphanumeric range
    // 14 bytes × modulo is cheaper than the getrandom syscall by far
    for b in buf[12..].iter_mut() {
        *b = CHARS[*b as usize % 62];
    }

    // SAFETY: buf contains only hex chars (0-9, a-f) + alphanumeric from CHARS
    // All are valid ASCII, therefore valid UTF-8. to_vec() copies stack→heap.
    unsafe { String::from_utf8_unchecked(buf.to_vec()) }
}

// Leetopt: Combined alphanumeric table for the random suffix
const CHARS: [u8; 62] = {
    let mut table = [0u8; 62];
    let mut i = 0;
    // Digits 0-9
    while i < 10 {
        table[i] = b'0' + i as u8;
        i += 1;
    }
    // A-Z
    while i < 36 {
        table[i] = b'A' + i as u8 - 10;
        i += 1;
    }
    // a-z
    while i < 62 {
        table[i] = b'a' + i as u8 - 36;
        i += 1;
    }
    table
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascending_length() {
        let id = ascending();
        assert_eq!(id.len(), 26);
    }

    #[test]
    fn test_descending_length() {
        let id = descending();
        assert_eq!(id.len(), 26);
    }

    #[test]
    fn test_ascending_order() {
        let a = from_timestamp(1000, false);
        let b = from_timestamp(2000, false);
        assert!(a < b);
    }

    #[test]
    fn test_descending_order() {
        let a = from_timestamp(1000, true);
        let b = from_timestamp(2000, true);
        assert!(b < a);
    }

    #[test]
    fn test_unique() {
        let ids: std::collections::HashSet<String> = (0..100).map(|_| ascending()).collect();
        assert_eq!(ids.len(), 100);
    }

    #[test]
    fn test_character_set() {
        let id = ascending();
        for c in id.chars() {
            assert!(
                c.is_ascii_alphanumeric(),
                "char '{}' not in alphanumeric set",
                c
            );
        }
    }
}
