//! Deterministic randomness.
//!
//! The engine owns its generator rather than depending on the `rand` ecosystem.
//! A replay is only worth storing if the same seed produces the same game a year
//! from now, and no crate upgrade can silently reshape a stream we implement
//! ourselves. This is `xoshiro256**` seeded through `SplitMix64` -- both are
//! public-domain reference designs with fixed, published output.
//!
//! Randomness is drawn from *substreams*, not one global sequence. A substream
//! is derived from the master seed plus a [`Domain`] and a salt, so a change in
//! one subsystem cannot shift another's draws: re-rolling your shop never
//! perturbs the combat that follows. That property is what makes a seed a
//! meaningful thing to share.

use serde::{Deserialize, Serialize};

/// An independent axis of randomness. Draws in one domain never disturb another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    /// Building the shared minion pool.
    Pool,
    /// Shop offers and rerolls.
    Shop,
    /// Who is paired against whom this round.
    Matchmaking,
    /// Coin flips and target selection inside a combat.
    Combat,
    /// Random targeting inside card effects.
    Effect,
    /// Hero and starting-state selection.
    Setup,
}

impl Domain {
    /// A fixed, never-reordered tag. Changing these values invalidates old seeds.
    const fn tag(self) -> u64 {
        match self {
            Domain::Pool => 0x506f_6f6c_0000_0001,
            Domain::Shop => 0x5368_6f70_0000_0002,
            Domain::Matchmaking => 0x4d61_7463_0000_0003,
            Domain::Combat => 0x436f_6d62_0000_0004,
            Domain::Effect => 0x4566_6663_0000_0005,
            Domain::Setup => 0x5365_7475_0000_0006,
        }
    }
}

/// The master seed for a game. Derives every substream the game uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seed(pub u64);

impl Seed {
    /// Derive the generator for one domain, distinguished by `salt` (typically a
    /// round number, seat index, or both mixed together).
    pub fn stream(self, domain: Domain, salt: u64) -> Rng {
        Rng::from_seed(mix(mix(self.0 ^ domain.tag()) ^ salt))
    }
}

impl std::fmt::Display for Seed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl std::str::FromStr for Seed {
    type Err = std::num::ParseIntError;

    /// Accepts either a hex seed as printed by [`Display`] or a plain decimal.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(hex) = s.strip_prefix("0x") {
            return u64::from_str_radix(hex, 16).map(Seed);
        }
        if s.len() == 16 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            return u64::from_str_radix(s, 16).map(Seed);
        }
        s.parse::<u64>().map(Seed)
    }
}

/// SplitMix64 finalizer -- turns a counter into well-distributed bits.
const fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut x = z;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// A deterministic `xoshiro256**` generator.
///
/// Cloning forks the stream: both copies produce the same subsequent values.
/// That is deliberate -- it lets callers speculatively evaluate an outcome
/// (an AI weighing a combat, say) without disturbing the real game's draws.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    /// Build a generator directly from a 64-bit seed.
    pub fn from_seed(seed: u64) -> Self {
        let mut z = seed;
        let mut s = [0u64; 4];
        for slot in &mut s {
            z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
            *slot = mix(z);
        }
        // xoshiro is undefined for an all-zero state; astronomically unlikely,
        // but cheap to rule out.
        if s == [0; 4] {
            s = [1, 2, 3, 4];
        }
        Rng { s }
    }

    /// The next 64 raw bits.
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// A uniform value in `0..n`, without modulo bias (Lemire's method).
    ///
    /// Returns 0 when `n == 0`, so callers indexing an empty collection get a
    /// harmless value rather than a panic; check emptiness first.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        let mut x = self.next_u64();
        let mut m = (x as u128).wrapping_mul(n as u128);
        let mut low = m as u64;
        if low < n {
            let threshold = n.wrapping_neg() % n;
            while low < threshold {
                x = self.next_u64();
                m = (x as u128).wrapping_mul(n as u128);
                low = m as u64;
            }
        }
        (m >> 64) as u64
    }

    /// A uniform value in the inclusive range `lo..=hi`. Panics if `lo > hi`.
    pub fn range_inclusive(&mut self, lo: i64, hi: i64) -> i64 {
        assert!(lo <= hi, "empty range {lo}..={hi}");
        lo + self.below((hi - lo) as u64 + 1) as i64
    }

    /// An even coin flip.
    pub fn flip(&mut self) -> bool {
        self.next_u64() >> 63 == 1
    }

    /// A uniformly chosen index into a collection of `len` items.
    pub fn index(&mut self, len: usize) -> Option<usize> {
        (len > 0).then(|| self.below(len as u64) as usize)
    }

    /// A uniformly chosen element.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        self.index(items.len()).map(|i| &items[i])
    }

    /// Fisher-Yates, iterating downward so the result depends only on the
    /// draw sequence and not on the slice's memory layout.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }

    /// Remove and return a uniformly chosen element, filling the gap with the
    /// last item. O(1), and deterministic given the vector's current order.
    pub fn swap_take<T>(&mut self, items: &mut Vec<T>) -> Option<T> {
        let i = self.index(items.len())?;
        Some(items.swap_remove(i))
    }

    /// Choose `n` distinct indices into `len` items, in the order drawn.
    pub fn sample_indices(&mut self, len: usize, n: usize) -> Vec<usize> {
        let mut pool: Vec<usize> = (0..len).collect();
        let take = n.min(len);
        let mut out = Vec::with_capacity(take);
        for _ in 0..take {
            let i = self.below(pool.len() as u64) as usize;
            out.push(pool.swap_remove(i));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::from_seed(42);
        let mut b = Rng::from_seed(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::from_seed(1);
        let mut b = Rng::from_seed(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn domains_are_independent() {
        let seed = Seed(0xdead_beef);
        let mut shop = seed.stream(Domain::Shop, 3);
        let mut combat = seed.stream(Domain::Combat, 3);
        assert_ne!(shop.next_u64(), combat.next_u64());
    }

    #[test]
    fn salt_separates_streams() {
        let seed = Seed(7);
        let mut r1 = seed.stream(Domain::Shop, 1);
        let mut r2 = seed.stream(Domain::Shop, 2);
        assert_ne!(r1.next_u64(), r2.next_u64());
    }

    #[test]
    fn below_stays_in_range_and_covers_it() {
        let mut rng = Rng::from_seed(99);
        let mut seen = [false; 6];
        for _ in 0..10_000 {
            let v = rng.below(6);
            assert!(v < 6);
            seen[v as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "every face of a d6 should appear");
    }

    #[test]
    fn below_zero_is_zero() {
        assert_eq!(Rng::from_seed(1).below(0), 0);
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut rng = Rng::from_seed(5);
        let mut items: Vec<u32> = (0..50).collect();
        rng.shuffle(&mut items);
        assert_ne!(items, (0..50).collect::<Vec<_>>());
        items.sort_unstable();
        assert_eq!(items, (0..50).collect::<Vec<_>>());
    }

    #[test]
    fn sample_indices_are_distinct() {
        let mut rng = Rng::from_seed(11);
        let picked = rng.sample_indices(10, 4);
        assert_eq!(picked.len(), 4);
        let mut sorted = picked.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 4);
    }

    #[test]
    fn sample_indices_saturates_at_len() {
        let mut rng = Rng::from_seed(11);
        assert_eq!(rng.sample_indices(3, 99).len(), 3);
    }

    #[test]
    fn clone_forks_the_stream() {
        let mut a = Rng::from_seed(123);
        a.next_u64();
        let mut b = a.clone();
        assert_eq!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn seed_roundtrips_through_text() {
        let seed = Seed(0x0123_4567_89ab_cdef);
        assert_eq!(seed.to_string(), "0123456789abcdef");
        assert_eq!("0123456789abcdef".parse::<Seed>().unwrap(), seed);
        assert_eq!("0x1f".parse::<Seed>().unwrap(), Seed(31));
        assert_eq!("31".parse::<Seed>().unwrap(), Seed(31));
    }
}
