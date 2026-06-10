//This module includes functionality translated from icao_filter.c

use std::sync::Mutex;

const ICAO_FILTER_SIZE: usize = 4096;
// Mask for wrapping a hash into [0, ICAO_FILTER_SIZE). Derived at compile time, so
// it costs nothing at runtime. `u32` to match `h` in the hash/probe code below.
const ICAO_FILTER_MASK: u32 = (ICAO_FILTER_SIZE as u32) - 1;
// The `& ICAO_FILTER_MASK` trick only maps cleanly onto array indices when the size
// is a power of two; guarantee that at compile time.
const _: () = assert!(ICAO_FILTER_SIZE.is_power_of_two());

pub const ICAO_FILTER_ADSB_NT: u32 = 1 << 25;

static ICAO_FILTER_A: Mutex<[u32; ICAO_FILTER_SIZE]> = Mutex::new([0; ICAO_FILTER_SIZE]);
static ICAO_FILTER_B: Mutex<[u32; ICAO_FILTER_SIZE]> = Mutex::new([0; ICAO_FILTER_SIZE]);

pub fn icao_flush() {
    let (mut a, mut b) = (ICAO_FILTER_A.lock().unwrap(), ICAO_FILTER_B.lock().unwrap());
    *a = [0; ICAO_FILTER_SIZE];
    *b = [0; ICAO_FILTER_SIZE];
}

/// Rotate the two-table ICAO filter so stale aircraft age out over time.
/// (Port of `icaoFilterExpire` from C dump1090.)
///
/// Intended semantics — called periodically (~every 60s) from a background thread:
///   1. Drop the oldest generation: clear table B.
///   2. Age the current generation: move table A's contents into B.
///   3. Start fresh: clear table A.
///
/// Why a swap and not `icao_flush`'s double-clear? `icao_filter_add` always writes
/// A, and `icao_filter_test` checks A *then* B. So after a rotation, an aircraft seen
/// in the previous cycle still answers `true` (via B) while A is rebuilt — giving each
/// track a 60–120s lifetime. A double-clear would wipe active aircraft every cycle.
///
/// LOCK ORDER: `icao_filter_test` (below) locks A then B. If you hold both locks at
/// once here, take them in that SAME order or you can deadlock against `test`.
pub fn icao_filter_expire() {
    let (mut a, mut b) = (ICAO_FILTER_A.lock().unwrap(), ICAO_FILTER_B.lock().unwrap());
    *b = std::mem::replace(&mut *a, [0; ICAO_FILTER_SIZE]);
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

    (hash as u32) & ICAO_FILTER_MASK
}

// The original function uses a integer return value, but it's used as a boolean
pub fn icao_filter_add(addr: u32) {
    let mut h: u32 = icao_hash(addr);
    let h0: u32 = h;
    if let Ok(mut icao_filter_a) = ICAO_FILTER_A.lock() {
        while (icao_filter_a[h as usize] != 0) && (icao_filter_a[h as usize] != addr) {
            h = (h + 1) & ICAO_FILTER_MASK;
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

    if let (Ok(icao_filter_a), Ok(icao_filter_b)) = (ICAO_FILTER_A.lock(), ICAO_FILTER_B.lock()) {
        'loop_a: while (icao_filter_a[h as usize] != 0) && (icao_filter_a[h as usize] != addr) {
            h = (h + 1) & ICAO_FILTER_MASK;
            if h == h0 {
                break 'loop_a;
            }
        }

        if icao_filter_a[h as usize] == addr {
            return true;
        }

        h = h0;

        'loop_b: while (icao_filter_b[h as usize] != 0) && (icao_filter_b[h as usize] != addr) {
            h = (h + 1) & ICAO_FILTER_MASK;
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
