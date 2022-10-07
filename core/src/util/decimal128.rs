use crate::set;
use bigdecimal::{BigDecimal, ParseBigDecimalError, FromPrimitive};
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::str::FromStr;

///
/// Adapted from https://github.com/mongodb/mongo-java-driver/blob/master/bson/src/main/org/bson/types/Decimal128.java
///
type Result<T> = std::result::Result<T, ParseBigDecimalError>;

const SERIAL_VERSION_UID: i64 = 4570973266503637887i64;

const INFINITY_MASK: i64 = 0x7800000000000000i64;
const NAN_MASK: i64 = 0x7c00000000000000i64;
const SIGN_BIT_MASK: i64 = 1i64 << 63;
const MIN_EXPONENT: i64 = -6176i64;
const MAX_EXPONENT: i32 = 6111;

const EXPONENT_OFFSET: i32 = 6176;
const MAX_BIT_LENGTH: i32 = 113;

lazy_static! {
    static ref NAN_STRINGS: HashSet<&'static str> = set! {"nan"};
    static ref NEGATIVE_NAN_STRINGS: HashSet<&'static str> = set! { "-nan"};
    static ref POSITIVE_INFINITY_STRINGS: HashSet<&'static str> =
        set! { "inf", "+inf", "infinity", "+infinity"};
    static ref NEGATIVE_INFINITY_STRINGS: HashSet<&'static str> = set! { "-inf", "-infinity"};

  ///
  /// A constant holding the positive infinity of type {@code Decimal128}.  It is equal to the value return by
  /// {@code Decimal128.valueOf("Infinity")}.
  ///
  pub static ref POSITIVE_INFINITY: Decimal128 = Decimal128::from_ieee_754_bid_encoding(INFINITY_MASK, 0);

  ///
  /// A constant holding the negative infinity of type {@code Decimal128}.  It is equal to the value return by
  /// {@code Decimal128.valueOf("-Infinity")}.
  ///
  pub static ref NEGATIVE_INFINITY: Decimal128 = Decimal128::from_ieee_754_bid_encoding(INFINITY_MASK | SIGN_BIT_MASK, 0);

  ///
  /// A constant holding a negative Not-a-Number (-NaN) value of type {@code Decimal128}.  It is equal to the value return by
  /// {@code Decimal128.valueOf("-NaN")}.
  ///
  pub static ref NEGATIVE_NAN: Decimal128 =   Decimal128::from_ieee_754_bid_encoding(NAN_MASK | SIGN_BIT_MASK, 0);

  ///
  /// A constant holding a Not-a-Number (NaN) value of type {@code Decimal128}.  It is equal to the value return by
  /// {@code Decimal128.valueOf("NaN")}.
  ///
  pub static ref NAN: Decimal128 =   Decimal128::from_ieee_754_bid_encoding(NAN_MASK, 0);

  ///
  /// A constant holding a positive zero value of type {@code Decimal128}.  It is equal to the value return by
  /// {@code Decimal128.valueOf("0")}.
  ///
  pub static ref POSITIVE_ZERO: Decimal128 =   Decimal128::from_ieee_754_bid_encoding(0x3040000000000000i64, 0x0000000000000000i64);

  ///
  /// A constant holding a negative zero value of type {@code Decimal128}.  It is equal to the value return by
  /// {@code Decimal128.valueOf("-0")}.
  ///
  pub static ref NEGATIVE_ZERO: Decimal128 =  Decimal128::  from_ieee_754_bid_encoding(-0x4FC0000000000000i64, 0x0000000000000000i64);
}

/// A binary integer decimal representation of a 128-bit decimal value, supporting 34 decimal digits of significand and an exponent range
/// of -6143 to +6144.
///
/// @see <a href="https://github.com/mongodb/specifications/blob/master/source/bson-decimal128/decimal128.rst">BSON Decimal128
/// specification</a>
/// @see <a href="https://en.wikipedia.org/wiki/Binary_Integer_Decimal">binary integer decimal</a>
/// @see <a href="https://en.wikipedia.org/wiki/Decimal128_floating-point_format">decimal128 floating-point format</a>
/// @see <a href="http://ieeexplore.ieee.org/document/4610935/">754-2008 - IEEE Standard for Floating-Point Arithmetic</a>
///
pub struct Decimal128 {
    high: i64,
    low: i64,
}

impl Decimal128 {
    ///
    /// Create an instance with the given high and low order bits representing this Decimal128 as an IEEE 754-2008 128-bit decimal
    /// floating point using the BID encoding scheme.
    ///
    /// @param high the high-order 64 bits
    /// @param low  the low-order 64 bits
    /// @return the Decimal128 value representing the given high and low order bits
    ///
    pub fn new(high: i64, low: i64) -> Decimal128 {
        Decimal128 { high, low }
    }

    ///
    /// Create an instance with the given high and low order bits representing this Decimal128 as an IEEE 754-2008 128-bit decimal
    /// floating point using the BID encoding scheme.
    ///
    /// @param high the high-order 64 bits
    /// @param low  the low-order 64 bits
    /// @return the Decimal128 value representing the given high and low order bits
    ///
    pub fn from_ieee_754_bid_encoding(high: i64, low: i64) -> Decimal128 {
        Self::new(high, low)
    }

    ///
    /// Returns a Decimal128 value representing the given String.
    ///
    /// @param value the Decimal128 value represented as a String
    /// @return the Decimal128 value representing the given String
    /// @throws NumberFormatException if the value is out of the Decimal128 range
    /// @see
    /// <a href="https://github.com/mongodb/specifications/blob/master/source/bson-decimal128/decimal128.rst#from-string-representation">
    ///     From-String Specification</a>
    ///
    pub fn parse(value: &str) -> Result<Decimal128> {
        let lower_case_value = value.to_lowercase();

        if NAN_STRINGS.contains(lower_case_value.as_str()) {
            return Ok(*NAN);
        }

        if NEGATIVE_NAN_STRINGS.contains(lower_case_value.as_str()) {
            return Ok(*NEGATIVE_NAN);
        }

        if POSITIVE_INFINITY_STRINGS.contains(lower_case_value.as_str()) {
            return Ok(*POSITIVE_INFINITY);
        }

        if NEGATIVE_INFINITY_STRINGS.contains(lower_case_value.as_str()) {
            return Ok(*NEGATIVE_INFINITY);
        }

        Decimal128::from_big_int(BigDecimal::from_str(value)?, value.chars().nth(0).ok_or(ParseBigDecimalError::Empty)? == '-')
    }

    // isNegative is necessary to detect -0, which can't be represented with a BigDecimal
    fn from_big_int(initial_value: BigDecimal, is_neg: bool) -> Result<Decimal128> {
        let mut local_high = 0;
        let mut local_low = 0;

        let value = Decimal128::clamp_and_round(initial_value);

        long exponent = -value.scale();

        if ((exponent < MIN_EXPONENT) || (exponent > MAX_EXPONENT)) {
            throw new AssertionError("Exponent is out of range for Decimal128 encoding: " + exponent); }

        if (value.unscaledValue().bitLength() > MAX_BIT_LENGTH) {
            throw new AssertionError("Unscaled roundedValue is out of range for Decimal128 encoding:" + value.unscaledValue());
        }

        BigInteger significand = value.unscaledValue().abs();
        int bitLength = significand.bitLength();

        for (int i = 0; i < Math.min(64, bitLength); i++) {
            if (significand.testBit(i)) {
                local_low |= 1L << i;
            }
        }

        for (int i = 64; i < bitLength; i++) {
            if (significand.testBit(i)) {
                local_high |= 1L << (i - 64);
            }
        }

        long biasedExponent = exponent + EXPONENT_OFFSET;

        local_high |= biasedExponent << 49;

        if (value.signum() == -1 || isNegative) {
            local_high |= SIGN_BIT_MASK;
        }

        high = local_high;
        low = local_low;
    }

    fn clamp_and_round(initial_value: BigDecimal) -> BigDecimal {
        let mut value = BigDecimal::from_u32(0).unwrap();
        let (bi, exponent) = initial_value.as_bigint_and_exponent();
        if (-initial_value.scale() > MAX_EXPONENT) {
            int diff = -initial_value.scale() - MAX_EXPONENT;
            if (initial_value.unscaledValue().equals(BIG_INT_ZERO)) {
                value = new BigDecimal(initial_value.unscaledValue(), -MAX_EXPONENT);
            } else if (diff + initial_value.precision() > 34) {
                throw new NumberFormatException("Exponent is out of range for Decimal128 encoding of " + initial_value);
            } else {
                BigInteger multiplier = BIG_INT_TEN.pow(diff);
                value = new BigDecimal(initial_value.unscaledValue().multiply(multiplier), initial_value.scale() + diff);
            }
        } else if (-exponent < MIN_EXPONENT) {
            // Increasing a very negative exponent may require decreasing precision, which is rounding
            // Only round exactly (by removing precision that is all zeroes).  An exception is thrown if the rounding would be inexact:
            // Exact:     .000...0011000  => 11000E-6177  => 1100E-6176  => .000001100
            // Inexact:   .000...0011001  => 11001E-6177  => 1100E-6176  => .000001100
            let diff = -exponent + MIN_EXPONENT;
            let undiscarded_precision = ensure_exact_rounding(initial_value, diff);
            let divisor = undiscarded_precision == 0 ? BIG_INT_ONE : BIG_INT_TEN.pow(diff);
            value = new BigDecimal(initial_value.unscaledValue().divide(divisor), initial_value.scale() - diff);
        } else {
            value = initial_value.round(DECIMAL128);
            int extraPrecision = initial_value.precision() - value.precision();
            if (extraPrecision > 0) {
                // Again, only round exactly
                ensureExactRounding(initial_value, extraPrecision);
            }
        }
        return value;
    }

    fn ensure_exact_rounding(initialValue: BigDecimal, extra_precision: i32) -> i32 {
          String significand = initialValue.digits);
//        int undiscardedPrecision = Math.max(0, significand.length() - extraPrecision);
//        for (int i = undiscardedPrecision; i < significand.length(); i++) {
//            if (significand.charAt(i) != '0') {
//                throw new NumberFormatException("Conversion to Decimal128 would require inexact rounding of " + initialValue);
//            }
//        }
//        return undiscardedPrecision;
//    }
}
//
//
//    ///
//     * Constructs a Decimal128 value representing the given long.
//     *
//     * @param value the Decimal128 value represented as a long
//     */
//    pub Decimal128(final long value) {
//        this(new BigDecimal(value, DECIMAL128));
//    }
//
//    ///
//     * Constructs a Decimal128 value representing the given BigDecimal.
//     *
//     * @param value the Decimal128 value represented as a BigDecimal
//     * @throws NumberFormatException if the value is out of the Decimal128 range
//     */
//    pub Decimal128(final BigDecimal value) {
//        this(value, value.signum() == -1);
//    }
//
//    private Decimal128(final long high, final long low) {
//        this.high = high;
//        this.low = low;
//    }
//
//
//
//
//    ///
//     * Gets the high-order 64 bits of the IEEE 754-2008 128-bit decimal floating point encoding for this Decimal128, using the BID encoding
//     * scheme.
//     *
//     * @return the high-order 64 bits of this Decimal128
//     */
//    pub long getHigh() {
//        return high;
//    }
//
//    ///
//     * Gets the low-order 64 bits of the IEEE 754-2008 128-bit decimal floating point encoding for this Decimal128, using the BID encoding
//     * scheme.
//     *
//     * @return the low-order 64 bits of this Decimal128
//     */
//    pub long getLow() {
//        return low;
//    }
//
//    ///
//     * Gets a BigDecimal that is equivalent to this Decimal128.
//     *
//     * @return a BigDecimal that is equivalent to this Decimal128
//     * @throws ArithmeticException if the Decimal128 value is NaN, Infinity, -Infinity, or -0, none of which can be represented as a
//     * BigDecimal
//     */
//    pub BigDecimal bigDecimalValue() {
//
//        if (isNaN()) {
//            throw new ArithmeticException("NaN can not be converted to a BigDecimal");
//        }
//
//        if (isInfinite()) {
//            throw new ArithmeticException("Infinity can not be converted to a BigDecimal");
//        }
//
//        BigDecimal bigDecimal = bigDecimalValueNoNegativeZeroCheck();
//
//        // If the BigDecimal is 0, but the Decimal128 is negative, that means we have -0.
//        if (isNegative() && bigDecimal.signum() == 0) {
//            throw new ArithmeticException("Negative zero can not be converted to a BigDecimal");
//        }
//
//        return bigDecimal;
//    }
//
//    // Make sure that the argument comes from a call to bigDecimalValueNoNegativeZeroCheck on this instance
//    private boolean hasDifferentSign(final BigDecimal bigDecimal) {
//        return isNegative() && bigDecimal.signum() == 0;
//    }
//
//    private boolean isZero(final BigDecimal bigDecimal) {
//        return !isNaN() && !isInfinite() && bigDecimal.compareTo(BigDecimal.ZERO) == 0;
//    }
//
//    private BigDecimal bigDecimalValueNoNegativeZeroCheck() {
//        int scale = -getExponent();
//
//        if (twoHighestCombinationBitsAreSet()) {
//            return BigDecimal.valueOf(0, scale);
//        }
//
//        return new BigDecimal(new BigInteger(isNegative() ? -1 : 1, getBytes()), scale);
//    }
//
//    // May have leading zeros.  Strip them before considering making this method pub
//    private byte[] getBytes() {
//        byte[] bytes = new byte[15];
//
//        long mask = 0x00000000000000ff;
//        for (int i = 14; i >= 7; i--) {
//            bytes[i] = (byte) ((low & mask) >>> ((14 - i) << 3));
//            mask = mask << 8;
//        }
//
//        mask = 0x00000000000000ff;
//        for (int i = 6; i >= 1; i--) {
//            bytes[i] = (byte) ((high & mask) >>> ((6 - i) << 3));
//            mask = mask << 8;
//        }
//
//        mask = 0x0001000000000000L;
//        bytes[0] = (byte) ((high & mask) >>> 48);
//        return bytes;
//    }
//
//    private int getExponent() {
//        if (twoHighestCombinationBitsAreSet()) {
//            return (int) ((high & 0x1fffe00000000000L) >>> 47) - EXPONENT_OFFSET;
//        } else {
//            return (int) ((high & 0x7fff800000000000L) >>> 49) - EXPONENT_OFFSET;
//        }
//    }
//
//    private boolean twoHighestCombinationBitsAreSet() {
//        return (high & 3L << 61) == 3L << 61;
//    }
//
//    ///
//     * Returns true if this Decimal128 is negative.
//     *
//     * @return true if this Decimal128 is negative
//     */
//    pub boolean isNegative() {
//        return (high & SIGN_BIT_MASK) == SIGN_BIT_MASK;
//    }
//
//    ///
//     * Returns true if this Decimal128 is infinite.
//     *
//     * @return true if this Decimal128 is infinite
//     */
//    pub boolean isInfinite() {
//        return (high & INFINITY_MASK) == INFINITY_MASK;
//    }
//
//    ///
//     * Returns true if this Decimal128 is finite.
//     *
//     * @return true if this Decimal128 is finite
//     */
//    pub boolean isFinite() {
//        return !isInfinite();
//    }
//
//    ///
//     * Returns true if this Decimal128 is Not-A-Number (NaN).
//     *
//     * @return true if this Decimal128 is Not-A-Number
//     */
//    pub boolean isNaN() {
//        return (high & NAN_MASK) == NAN_MASK;
//    }
//
//
//    @Override
//    pub int compareTo(final Decimal128 o) {
//        if (isNaN()) {
//            return o.isNaN() ? 0 : 1;
//        }
//        if (isInfinite()) {
//            if (isNegative()) {
//                if (o.isInfinite() && o.isNegative()) {
//                    return 0;
//                } else {
//                    return -1;
//                }
//            } else {
//                if (o.isNaN()) {
//                    return -1;
//                } else if (o.isInfinite() && !o.isNegative()) {
//                    return 0;
//                } else {
//                    return 1;
//                }
//            }
//        }
//        BigDecimal bigDecimal = bigDecimalValueNoNegativeZeroCheck();
//        BigDecimal otherBigDecimal = o.bigDecimalValueNoNegativeZeroCheck();
//
//        if (isZero(bigDecimal) && o.isZero(otherBigDecimal)) {
//            if (hasDifferentSign(bigDecimal)) {
//                if (o.hasDifferentSign(otherBigDecimal)) {
//                    return 0;
//                }
//                else {
//                    return -1;
//                }
//            } else if (o.hasDifferentSign(otherBigDecimal)) {
//                return 1;
//            }
//        }
//
//        if (o.isNaN()) {
//            return -1;
//        } else if (o.isInfinite()) {
//            if (o.isNegative()) {
//                return 1;
//            } else {
//                return -1;
//            }
//        } else {
//            return bigDecimal.compareTo(otherBigDecimal);
//        }
//    }
//
//    ///
//     * Converts this {@code Decimal128} to a {@code int}. This conversion is analogous to the <i>narrowing primitive conversion</i> from
//     * {@code double} to {@code int} as defined in <cite>The Java&trade; Language Specification</cite>: any fractional part of this
//     * {@code Decimal128} will be discarded, and if the resulting integral value is too big to fit in a {@code int}, only the
//     * low-order 32 bits are returned. Note that this conversion can lose information about the overall magnitude and precision of this
//     * {@code Decimal128} value as well as return a result with the opposite sign. Note that {@code #NEGATIVE_ZERO} is converted to
//     * {@code 0}.
//     *
//     * @return this {@code Decimal128} converted to a {@code int}.
//     * @since 3.10
//     */
//    @Override
//    pub int intValue() {
//        return (int) doubleValue();
//    }
//
//    ///
//     * Converts this {@code Decimal128} to a {@code long}. This conversion is analogous to the <i>narrowing primitive conversion</i> from
//     * {@code double} to {@code long} as defined in <cite>The Java&trade; Language Specification</cite>: any fractional part of this
//     * {@code Decimal128} will be discarded, and if the resulting integral value is too big to fit in a {@code long}, only the
//     * low-order 64 bits are returned. Note that this conversion can lose information about the overall magnitude and precision of this
//     * {@code Decimal128} value as well as return a result with the opposite sign. Note that {@code #NEGATIVE_ZERO} is converted to
//     * {@code 0L}.
//     *
//     * @return this {@code Decimal128} converted to a {@code long}.
//     * @since 3.10
//     */
//    @Override
//    pub long longValue() {
//        return (long) doubleValue();
//    }
//
//    ///
//     * Converts this {@code Decimal128} to a {@code float}. This conversion is similar to the <i>narrowing primitive conversion</i> from
//     * {@code double} to {@code float} as defined in <cite>The Java&trade; Language Specification</cite>: if this {@code Decimal128} has
//     * too great a magnitude to represent as a {@code float}, it will be converted to {@link Float#NEGATIVE_INFINITY} or
//     * {@link Float#POSITIVE_INFINITY} as appropriate.  Note that even when the return value is finite, this conversion can lose
//     * information about the precision of the {@code Decimal128} value. Note that {@code #NEGATIVE_ZERO} is converted to {@code 0.0f}.
//     *
//     * @return this {@code Decimal128} converted to a {@code float}.
//     * @since 3.10
//     */
//    @Override
//    pub float floatValue() {
//        return (float) doubleValue();
//    }
//
//    ///
//     * Converts this {@code Decimal128} to a {@code double}. This conversion is similar to the <i>narrowing primitive conversion</i> from
//     * {@code double} to {@code float} as defined in <cite>The Java&trade; Language Specification</cite>: if this {@code Decimal128} has
//     * too great a magnitude to represent as a {@code double}, it will be converted to {@link Double#NEGATIVE_INFINITY} or
//     * {@link Double#POSITIVE_INFINITY} as appropriate.  Note that even when the return value is finite, this conversion can lose
//     * information about the precision of the {@code Decimal128} value. Note that {@code #NEGATIVE_ZERO} is converted to {@code 0.0d}.
//     *
//     * @return this {@code Decimal128} converted to a {@code double}.
//     * @since 3.10
//     */
//    @Override
//    pub double doubleValue() {
//        if (isNaN()) {
//            return Double.NaN;
//        }
//        if (isInfinite()) {
//            if (isNegative()) {
//                return Double.NEGATIVE_INFINITY;
//            } else {
//                return Double.POSITIVE_INFINITY;
//            }
//        }
//
//        BigDecimal bigDecimal = bigDecimalValueNoNegativeZeroCheck();
//
//        if (hasDifferentSign(bigDecimal)) {
//            return -0.0d;
//        }
//
//        return bigDecimal.doubleValue();
//    }
//
//    ///
//     * Returns true if the encoded representation of this instance is the same as the encoded representation of {@code o}.
//     * <p>
//     * One consequence is that, whereas {@code Double.NaN != Double.NaN},
//     * {@code new Decimal128("NaN").equals(new Decimal128("NaN")} returns true.
//     * </p>
//     * <p>
//     * Another consequence is that, as with BigDecimal, {@code new Decimal128("1.0").equals(new Decimal128("1.00")} returns false,
//     * because the precision is not the same and therefore the representation is not the same.
//     * </p>
//     *
//     * @param o the object to compare for equality
//     * @return true if the instances are equal
//     */
//    @Override
//    pub boolean equals(final Object o) {
//        if (this == o) {
//            return true;
//        }
//        if (o == null || getClass() != o.getClass()) {
//            return false;
//        }
//
//        Decimal128 that = (Decimal128) o;
//
//        if (high != that.high) {
//            return false;
//        }
//        if (low != that.low) {
//            return false;
//        }
//
//        return true;
//    }
//
//    @Override
//    pub int hashCode() {
//        int result = (int) (low ^ (low >>> 32));
//        result = 31 * result + (int) (high ^ (high >>> 32));
//        return result;
//    }
//
//    ///
//     * Returns the String representation of the Decimal128 value.
//     *
//     * @return the String representation
//     * @see <a href="https://github.com/mongodb/specifications/blob/master/source/bson-decimal128/decimal128.rst#to-string-representation">
//     *     To-String Sprecification</a>
//     */
//    @Override
//    pub String toString() {
//        if (isNaN()) {
//            return "NaN";
//        }
//        if (isInfinite()) {
//            if (isNegative()) {
//                return "-Infinity";
//            } else {
//                return "Infinity";
//            }
//        }
//        return toStringWithBigDecimal();
//    }
//
//    private String toStringWithBigDecimal() {
//        StringBuilder buffer = new StringBuilder();
//
//        BigDecimal bigDecimal = bigDecimalValueNoNegativeZeroCheck();
//        String significand = bigDecimal.unscaledValue().abs().toString();
//
//        if (isNegative()) {
//            buffer.append('-');
//        }
//
//        int exponent = -bigDecimal.scale();
//        int adjustedExponent = exponent + (significand.length() - 1);
//        if (exponent <= 0 && adjustedExponent >= -6) {
//            if (exponent == 0) {
//                buffer.append(significand);
//            } else {
//                int pad = -exponent - significand.length();
//                if (pad >= 0) {
//                    buffer.append('0');
//                    buffer.append('.');
//                    for (int i = 0; i < pad; i++) {
//                        buffer.append('0');
//                    }
//                    buffer.append(significand, 0, significand.length());
//                } else {
//                    buffer.append(significand, 0, -pad);
//                    buffer.append('.');
//                    buffer.append(significand, -pad, -pad - exponent);
//                }
//            }
//        } else {
//            buffer.append(significand.charAt(0));
//            if (significand.length() > 1) {
//                buffer.append('.');
//                buffer.append(significand, 1, significand.length());
//            }
//            buffer.append('E');
//            if (adjustedExponent > 0) {
//                buffer.append('+');
//            }
//            buffer.append(adjustedExponent);
//        }
//        return buffer.toString();
//    }
//}
