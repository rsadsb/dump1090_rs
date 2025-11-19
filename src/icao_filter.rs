//This module includes functionality translated from icao_filter.c

use std::sync::RwLock;

const ICAO_FILTER_SIZE: u32 = 4096;
pub const ICAO_FILTER_ADSB_NT: u32 = 1 << 25;

static ICAO_FILTER_A: RwLock<[u32; 4096]> = RwLock::new([0; 4096]);
static ICAO_FILTER_B: RwLock<[u32; 4096]> = RwLock::new([0; 4096]);

pub fn icao_flush() {
    let mut i = ICAO_FILTER_A.write().unwrap();
    *i = [0; 4096];

    let mut i = ICAO_FILTER_B.write().unwrap();
    *i = [0; 4096];
}

pub fn icao_hash(a32: u32) -> u32 // icao_filter.c:38
{
    let a: u64 = u64::from(a32);

    // Jenkins one-at-a-time hash, unrolled for 3 bytes
    let mut hash: u64 = 0;

    hash += a & 0xff;
    hash += hash << 10;
    hash ^= hash >> 6;

    hash += (a >> 8) & 0xff;
    hash += hash << 10;
    hash ^= hash >> 6;

    hash += (a >> 16) & 0xff;
    hash += hash << 10;
    hash ^= hash >> 6;

    hash += hash << 3;
    hash ^= hash >> 11;
    hash += hash << 15;

    (hash as u32) & (ICAO_FILTER_SIZE - 1)
}

// The original function uses a integer return value, but it's used as a boolean
pub fn icao_filter_add(addr: u32) {
    let mut h: u32 = icao_hash(addr);
    let h0: u32 = h;
    if let Ok(mut icao_filter_a) = ICAO_FILTER_A.write() {
        while (icao_filter_a[h as usize] != 0) && (icao_filter_a[h as usize] != addr) {
            h = (h + 1) & (ICAO_FILTER_SIZE - 1);
            if h == h0 {
                eprintln!("icao24 hash table full");
                return;
            }
        }

        if icao_filter_a[h as usize] == 0 {
            icao_filter_a[h as usize] = addr;
        }
    }
}

// The original function uses a integer return value, but it's used as a boolean
pub fn icao_filter_test(addr: u32) -> bool // icao_filter.c:96
{
    let mut h: u32 = icao_hash(addr);
    let h0: u32 = h;

    if let (Ok(icao_filter_a), Ok(icao_filter_b)) = (ICAO_FILTER_A.read(), ICAO_FILTER_B.read()) {
        'loop_a: while (icao_filter_a[h as usize] != 0) && (icao_filter_a[h as usize] != addr) {
            h = (h + 1) & (ICAO_FILTER_SIZE - 1);
            if h == h0 {
                break 'loop_a;
            }
        }

        if icao_filter_a[h as usize] == addr {
            return true;
        }

        h = h0;

        'loop_b: while (icao_filter_b[h as usize] != 0) && (icao_filter_b[h as usize] != addr) {
            h = (h + 1) & (ICAO_FILTER_SIZE - 1);
            if h == h0 {
                break 'loop_b;
            }
        }

        if icao_filter_b[h as usize] == addr {
            return true;
        }
    }

    false
}
