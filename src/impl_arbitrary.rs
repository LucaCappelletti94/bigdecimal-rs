//!
//! Support for `arbitrary` crate
//!
//! Implements the `Arbitrary` trait for `BigDecimal`, allowing it to be
//! used with fuzzing tools like `cargo-fuzz` and `libfuzzer`.
//!

use crate::BigDecimal;
use arbitrary::{Arbitrary, Unstructured, Result as ArbitraryResult};

/// Maximum absolute value of scale to generate
///
/// This limit prevents memory allocation failures that can occur when
/// generating BigDecimals with extremely large scales, which would
/// result in very long string representations.
const SCALE_LIMIT: i64 = 1_000_000;

impl<'a> Arbitrary<'a> for BigDecimal {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let int_val = num_bigint::BigInt::arbitrary(u)?;
        let scale = i64::arbitrary(u)? % SCALE_LIMIT;
        Ok(BigDecimal::new(int_val, scale))
    }

    fn arbitrary_take_rest(mut u: Unstructured<'a>) -> ArbitraryResult<Self> {
        // Use arbitrary for scale first (fixed size), then take rest for BigInt
        let scale = i64::arbitrary(&mut u)? % SCALE_LIMIT;
        let int_val = num_bigint::BigInt::arbitrary_take_rest(u)?;
        Ok(BigDecimal::new(int_val, scale))
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        arbitrary::size_hint::and(
            num_bigint::BigInt::size_hint(depth),
            i64::size_hint(depth),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrary_bigdecimal() {
        // Test that we can generate BigDecimals from arbitrary bytes
        let data: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let mut u = Unstructured::new(data);
        let result = BigDecimal::arbitrary(&mut u);
        assert!(result.is_ok());
    }

    #[test]
    fn test_arbitrary_bigdecimal_scale_is_limited() {
        // Test with data that would produce a large scale value
        // The scale should be limited by SCALE_LIMIT
        let data: &[u8] = &[0xFF; 32];
        let mut u = Unstructured::new(data);
        if let Ok(bd) = BigDecimal::arbitrary(&mut u) {
            assert!(bd.fractional_digit_count().abs() < SCALE_LIMIT);
        }
    }

    #[test]
    fn test_arbitrary_take_rest() {
        let data: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
        let u = Unstructured::new(data);
        let result = BigDecimal::arbitrary_take_rest(u);
        assert!(result.is_ok());
    }

    #[test]
    fn test_size_hint() {
        let (min, max) = BigDecimal::size_hint(0);
        // Should have some minimum size requirement
        assert!(min > 0 || max.is_some());
    }
}
