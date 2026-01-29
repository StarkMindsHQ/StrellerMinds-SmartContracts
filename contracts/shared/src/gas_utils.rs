use alloc::vec::Vec;
use soroban_sdk::Env;

/// Utility helpers for gas-optimized patterns.
///
/// This module contains small, well-tested pure helpers used by contracts
/// to pack boolean flags into compact integers, perform bit ops and split
/// large collections into chunks for batched storage writes. Keep helpers
/// tiny and deterministic so they can be inlined by the optimizer.
pub struct GasUtils;

impl GasUtils {
    /// Pack up to 128 boolean flags into a single `u128` value.
    /// The first flag maps to the least significant bit.
    pub fn pack_bools(flags: &[bool]) -> u128 {
        let mut bits: u128 = 0;
        let mut shift: u32 = 0;
        for &f in flags.iter().take(128) {
            if f {
                bits |= 1u128 << shift;
            }
            shift += 1;
        }
        bits
    }

    /// Unpack `count` booleans from a u128 produced by `pack_bools`.
    pub fn unpack_bools(mut bits: u128, count: usize) -> Vec<bool> {
        let mut out: Vec<bool> = Vec::new();
        let n = if count > 128 { 128 } else { count };
        for _ in 0..n {
            out.push((bits & 1u128) == 1u128);
            bits >>= 1;
        }
        out
    }

    /// Set a single bit at `idx` (0-based) in the packed u128.
    pub fn set_bit(bits: u128, idx: usize) -> u128 {
        if idx >= 128 {
            return bits;
        }
        bits | (1u128 << (idx as u32))
    }

    /// Clear a single bit at `idx` (0-based) in the packed u128.
    pub fn clear_bit(bits: u128, idx: usize) -> u128 {
        if idx >= 128 {
            return bits;
        }
        bits & !(1u128 << (idx as u32))
    }

    /// Chunk a `total` length into a vector of (start, len) ranges with at most
    /// `chunk_size` elements per range. Useful for batching storage writes.
    pub fn chunk_ranges(total: usize, chunk_size: usize) -> Vec<(u32, u32)> {
        let mut out: Vec<(u32, u32)> = Vec::new();
        if chunk_size == 0 || total == 0 {
            return out;
        }
        let mut start: usize = 0;
        while start < total {
            let len = if start + chunk_size > total {
                total - start
            } else {
                chunk_size
            };
            out.push((start as u32, len as u32));
            start += len;
        }
        out
    }

    /// Lazy-load pattern: attempt to get a stored value (via closure) and only
    /// compute the fallback if missing. This is a tiny helper that takes an
    /// Env reference so callers can keep storage access tight.
    ///
    /// Usage: if env.storage().instance().has(&key) { get } else { compute and set }
    /// Provide this pattern inline in contracts to avoid repeated storage reads.
    pub fn lazy_compute<T, FGet, FCompute>(env: &Env, has: FGet, compute: FCompute) -> T
    where
        FGet: Fn(&Env) -> Option<T>,
        FCompute: Fn(&Env) -> T,
    {
        if let Some(v) = has(env) {
            v
        } else {
            compute(env)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GasUtils;

    #[test]
    fn pack_unpack_roundtrip() {
        let flags = [true, false, true, true, false];
        let packed = GasUtils::pack_bools(&flags);
        let unpacked = GasUtils::unpack_bools(packed, flags.len());
        assert_eq!(unpacked.len(), flags.len());
        for (i, &b) in flags.iter().enumerate() {
            assert_eq!(unpacked[i], b);
        }
    }
}
