//This module includes functionality translated from icao_filter.c

use std::sync::Mutex;

const ICAO_FILTER_SIZE: u32 = 4096;

lazy_static! {
    static ref ICAO_FILTER_A: Mutex<Vec<u32>> = Mutex::new(vec![0; 4096]);
    static ref ICAO_FILTER_B: Mutex<Vec<u32>> = Mutex::new(vec![0; 4096]);
}

pub fn icao_hash(a32: u32) -> u32 // icao_filter.c:38
{
    let a: u64 = a32 as u64;

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

pub fn icao_filter_add(addr: u32) {
    if let Ok(mut icao_filter_active) = ICAO_FILTER_A.lock() {
        // TODO: switch filters like the C code instead of using the same one every time
        {
            let mut h: u32 = icao_hash(addr);
            let h0: u32 = icao_hash(addr);

            while (icao_filter_active[h as usize] != 0) && (icao_filter_active[h as usize] != addr)
            {
                h = (h + 1) & (ICAO_FILTER_SIZE - 1);
                if h == h0 {
                    eprintln!("ICAO hash table full, increase ICAO_FILTER_SIZE");
                    return;
                }
            }
            if icao_filter_active[h as usize] == 0 {
                icao_filter_active[h as usize] = addr;
            }
        }

        // also add with a zeroed top byte, for handling DF20/21 with Data Parity
        {
            let mut h: u32 = icao_hash(addr & 0x00ffff);
            let h0: u32 = icao_hash(addr & 0x00ffff);
            while (icao_filter_active[h as usize] != 0)
                && ((icao_filter_active[h as usize] & 0x00ffff) != (addr & 0x00ffff))
            {
                h = (h + 1) & (ICAO_FILTER_SIZE - 1);
                if h == h0 {
                    eprintln!("ICAO hash table full, increase ICAO_FILTER_SIZE\n");
                    return;
                }
            }
            if icao_filter_active[h as usize] == 0 {
                icao_filter_active[h as usize] = addr;
            }
        }
    }
}

// The original function uses a integer return value, but it's used as a boolean
pub fn icao_filter_test(addr: u32) -> bool // icao_filter.c:96
{
    let mut h: u32 = icao_hash(addr);
    let h0: u32 = icao_hash(addr);

    if let (Ok(icao_filter_a), Ok(icao_filter_b)) = (ICAO_FILTER_A.lock(), ICAO_FILTER_B.lock()) {
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
