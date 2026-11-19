//! The crate's own PRNG and the entropy seam used by every generator.
//!
//! The algorithm is frozen: splitmix64 expands the seed, xoshiro256** produces
//! the stream. Changing either is a version event, not a refactor.

use crate::seed::Seed;

const GOLDEN_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

const fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(GOLDEN_GAMMA);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

/// A yes-rate in basis points, `0..=10_000`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Share(u16);

impl Share {
    pub(crate) const MAX: u16 = 10_000;

    /// Builds a rate from a whole percentage, saturating at 100%.
    #[must_use]
    pub(crate) const fn percent(percent: u16) -> Self {
        let basis_points = percent.saturating_mul(100);
        Self(if basis_points > Self::MAX { Self::MAX } else { basis_points })
    }

    #[must_use]
    pub(crate) const fn basis_points(self) -> u16 {
        self.0
    }
}

/// The deterministic source of every generated value.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RngEntropy {
    s0: u64,
    s1: u64,
    s2: u64,
    s3: u64,
}

impl RngEntropy {
    /// Expands a seed into the xoshiro256** state.
    #[must_use]
    pub(crate) const fn from_seed(seed: Seed) -> Self {
        let mut state = seed.into_inner();
        let s0 = splitmix64(&mut state);
        let s1 = splitmix64(&mut state);
        let s2 = splitmix64(&mut state);
        let s3 = splitmix64(&mut state);
        Self { s0, s1, s2, s3 }
    }

    const fn next_u64(&mut self) -> u64 {
        let result = self.s1.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let shifted = self.s1 << 17_u32;

        self.s2 ^= self.s0;
        self.s3 ^= self.s1;
        self.s1 ^= self.s2;
        self.s0 ^= self.s3;
        self.s2 ^= shifted;
        self.s3 = self.s3.rotate_left(45);

        result
    }
}

/// The seam through which every profile draws its values.
///
/// `arbitrary` and `proptest` adapters will call the same methods, so the
/// generator has exactly one path.
pub(crate) trait Entropy {
    /// Returns a value in `0..upper`; zero means zero.
    fn below(&mut self, upper: u32) -> u32;

    /// Returns true with the given basis-point probability.
    fn bit(&mut self, yes_rate: Share) -> bool {
        if yes_rate.basis_points() >= Share::MAX {
            return true;
        }
        if yes_rate.basis_points() == 0 {
            return false;
        }
        self.below(u32::from(Share::MAX)) < u32::from(yes_rate.basis_points())
    }

    /// Picks one item, or `None` for an empty slice.
    fn pick<'items, T>(&mut self, items: &'items [T]) -> Option<&'items T> {
        let upper = u32::try_from(items.len()).ok()?;
        if upper == 0 {
            return None;
        }
        let index = usize::try_from(self.below(upper)).ok()?;
        items.get(index)
    }
}

impl Entropy for RngEntropy {
    fn below(&mut self, upper: u32) -> u32 {
        if upper == 0 {
            return 0;
        }

        // Lemire's multiply-shift; u64 * u32 always fits u128.
        let product = u128::from(self.next_u64()).wrapping_mul(u128::from(upper));
        u32::try_from(product >> 64).unwrap_or(0)
    }
}

/// An entropy source that reads its values from a byte slice.
///
/// One draw consumes eight bytes, little-endian. A short tail is zero-padded;
/// once the slice is exhausted every draw is zero. That keeps generation total:
/// an `arbitrary` or property-test input may end in the middle of a request,
/// and the contract is to produce a request anyway, never to panic.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct BytesEntropy<'bytes> {
    bytes: &'bytes [u8],
    cursor: usize,
}

#[cfg(test)]
impl<'bytes> BytesEntropy<'bytes> {
    /// Wraps a byte slice as an entropy source.
    #[must_use]
    pub(crate) const fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    /// Returns the number of bytes no draw has consumed yet.
    #[must_use]
    pub(crate) const fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.cursor)
    }

    fn next_u64(&mut self) -> u64 {
        let mut word = [0_u8; 8];
        let end = self.cursor.saturating_add(word.len()).min(self.bytes.len());

        if let Some(chunk) = self.bytes.get(self.cursor..end) {
            if let Some(head) = word.get_mut(..chunk.len()) {
                head.copy_from_slice(chunk);
            }
            self.cursor = end;
        }

        u64::from_le_bytes(word)
    }
}

#[cfg(test)]
impl Entropy for BytesEntropy<'_> {
    fn below(&mut self, upper: u32) -> u32 {
        if upper == 0 {
            return 0;
        }

        let product = u128::from(self.next_u64()).wrapping_mul(u128::from(upper));
        u32::try_from(product >> 64).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::{BytesEntropy, Entropy, RngEntropy, Share, splitmix64};
    use crate::seed::Seed;

    #[test]
    fn golden_vectors_are_frozen() {
        let cases = [
            (
                0_u64,
                [
                    0x99EC_5F36_CB75_F2B4,
                    0xBF6E_1F78_4956_452A,
                    0x1A5F_849D_4933_E6E0,
                    0x6AA5_94F1_262D_2D2C,
                ],
            ),
            (
                42,
                [
                    0x1578_0B2E_0C2E_C716,
                    0x6104_D986_6D11_3A7E,
                    0xAE17_5332_39E4_99A1,
                    0xECB8_AD47_03B3_60A1,
                ],
            ),
            (
                u64::MAX,
                [
                    0x8F55_20D5_2A7E_AD08,
                    0xC476_A018_CAA1_802D,
                    0x81DE_31C0_D260_469E,
                    0xBF65_8D7E_065F_3C2F,
                ],
            ),
        ];

        for (seed, expected) in cases {
            let mut rng = RngEntropy::from_seed(Seed::new(seed));
            for value in expected {
                assert_eq!(rng.next_u64(), value);
            }
        }
    }

    #[test]
    fn splitmix64_is_stable() {
        let mut state = 42;
        assert_eq!(splitmix64(&mut state), 0xBDD7_3226_2FEB_6E95);
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = RngEntropy::from_seed(Seed::new(7));
        for _ in 0_u32..1_000 {
            assert!(rng.below(10) < 10);
        }
        assert_eq!(rng.below(0), 0);
    }

    #[test]
    fn rates_have_fixed_ends() {
        let mut rng = RngEntropy::from_seed(Seed::new(0));
        for _ in 0_u32..100 {
            assert!(!rng.bit(Share::percent(0)));
            assert!(rng.bit(Share::percent(100)));
        }
    }

    #[test]
    fn picks_from_a_slice_and_handles_empty() {
        let mut rng = RngEntropy::from_seed(Seed::new(1));
        let values = ["a", "b", "c"];
        assert!(rng.pick(&values).is_some());
        assert!(rng.pick::<u8>(&[]).is_none());
    }

    #[test]
    fn byte_entropy_pins_its_draws() {
        let mut entropy = BytesEntropy::new(&[0, 1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(entropy.below(u32::MAX), 0x0706_0503);
        assert_eq!(entropy.remaining(), 0);
    }

    #[test]
    fn byte_entropy_zero_pads_a_short_tail() {
        let mut entropy = BytesEntropy::new(&[0, 1, 2]);
        assert_eq!(entropy.remaining(), 3);

        assert_eq!(entropy.below(u32::MAX), 0);
        assert_eq!(entropy.remaining(), 0);
        assert_eq!(entropy.below(u32::MAX), 0);
    }

    #[test]
    fn byte_entropy_is_zero_after_exhaustion() {
        let mut entropy = BytesEntropy::new(&[]);

        for _ in 0_u32..100 {
            assert_eq!(entropy.below(10), 0);
        }
        assert_eq!(entropy.remaining(), 0);
    }

    #[test]
    fn fixed_rates_and_empty_picks_do_not_consume_bytes() {
        let mut entropy = BytesEntropy::new(&[1, 2, 3, 4, 5, 6, 7, 8]);

        assert!(!entropy.bit(Share::percent(0)));
        assert!(entropy.bit(Share::percent(100)));
        assert!(entropy.pick::<u8>(&[]).is_none());
        assert_eq!(entropy.below(0), 0);
        assert_eq!(entropy.remaining(), 8);
    }

    #[test]
    fn byte_picks_stay_in_bounds() {
        let mut entropy = BytesEntropy::new(&[0xFF; 8]);
        let values = ["a", "b", "c"];

        assert!(entropy.pick(&values).is_some());
        assert_eq!(entropy.remaining(), 0);
    }
}
