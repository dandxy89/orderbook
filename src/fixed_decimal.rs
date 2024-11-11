use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

#[cfg(feature = "serde")]
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedDecimal {
    raw: i64,
}

#[cfg(feature = "serde")]
impl Serialize for FixedDecimal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
struct FixedDecimalVisitor;

#[cfg(feature = "serde")]
#[allow(clippy::needless_lifetimes)]
impl<'de> Visitor<'de> for FixedDecimalVisitor {
    type Value = FixedDecimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a decimal number as a string or integer")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FixedDecimal::from_int(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value <= i64::MAX as u64 {
            Ok(FixedDecimal::from_int(value as i64))
        } else {
            Err(E::custom("integer too large"))
        }
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FixedDecimal::from_f64(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        FixedDecimal::from_str(value).map_err(E::custom)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for FixedDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FixedDecimalVisitor)
    }
}

#[allow(dead_code)]
impl FixedDecimal {
    const SCALE: i64 = 12;
    const SCALE_FACTOR: i64 = 1_000_000_000_000;
    const ZERO: FixedDecimal = FixedDecimal { raw: 0 };
    const ONE: FixedDecimal = FixedDecimal { raw: Self::SCALE_FACTOR };
    const MAX: FixedDecimal = FixedDecimal { raw: 1_000_000 * Self::SCALE_FACTOR };
    const MIN: FixedDecimal = FixedDecimal { raw: -1_000_000 * Self::SCALE_FACTOR };

    #[inline(always)]
    pub const fn new(raw: i64) -> Self {
        Self { raw }
    }

    #[inline(always)]
    pub const fn from_int(value: i64) -> Self {
        if value.abs() > 1_000_000 {
            panic!("Whole number exceeds maximum value of 1 million");
        }
        Self { raw: value * Self::SCALE_FACTOR }
    }

    #[inline(always)]
    pub fn from_parts(whole: i64, decimal: u32) -> Self {
        if whole.abs() > 1_000_000 {
            panic!("Whole number exceeds maximum value of 1 million");
        }
        let mut raw = whole * Self::SCALE_FACTOR;
        // Convert the decimal part to the correct scale
        let decimal_digits = if decimal == 0 { 0 } else { (decimal as f64).log10().floor() as u32 + 1 };
        // Scale the decimal part to match our precision
        let scale_multiplier =
            if decimal_digits < Self::SCALE as u32 { 10_i64.pow(Self::SCALE as u32 - decimal_digits) } else { 1 };
        let decimal_contribution = if decimal_digits <= Self::SCALE as u32 {
            (decimal as i64) * scale_multiplier
        } else {
            (decimal as i64) / 10_i64.pow(decimal_digits - Self::SCALE as u32)
        };
        raw += if raw < 0 { -decimal_contribution } else { decimal_contribution };
        Self { raw }
    }

    #[inline(always)]
    pub fn to_f64(self) -> f64 {
        self.raw as f64 / Self::SCALE_FACTOR as f64
    }

    #[inline(always)]
    pub const fn raw_value(self) -> i64 {
        self.raw
    }

    #[inline(always)]
    pub fn round_to(self, places: u32) -> Self {
        let factor = 10_i64.pow(Self::SCALE as u32 - places);
        Self { raw: (self.raw / factor) * factor }
    }

    #[inline(always)]
    pub const fn is_zero(self) -> bool {
        self.raw == 0
    }

    #[inline(always)]
    pub const fn is_negative(self) -> bool {
        self.raw < 0
    }

    #[inline(always)]
    pub fn abs(self) -> Self {
        Self { raw: self.raw.abs() }
    }

    #[inline(always)]
    pub fn from_f64(value: f64) -> Self {
        if value.is_nan() {
            return Self::ZERO;
        }
        if value.is_infinite() {
            return if value.is_sign_positive() { Self::MAX } else { Self::MIN };
        }
        let scaled = value * Self::SCALE_FACTOR as f64;
        if scaled >= i64::MAX as f64 {
            return Self::MAX;
        }
        if scaled <= i64::MIN as f64 {
            return Self::MIN;
        }
        let raw = scaled.round() as i64;
        Self { raw }
    }

    #[inline(always)]
    pub fn from_f32(value: f32) -> Self {
        Self::from_f64(value as f64)
    }
}

impl Add for FixedDecimal {
    type Output = Self;

    #[inline(always)]
    fn add(self, other: Self) -> Self {
        Self { raw: self.raw.saturating_add(other.raw) }
    }
}

impl Sub for FixedDecimal {
    type Output = Self;

    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        Self { raw: self.raw.saturating_sub(other.raw) }
    }
}

impl Mul for FixedDecimal {
    type Output = Self;

    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        let result = (self.raw as i128 * other.raw as i128) / Self::SCALE_FACTOR as i128;
        Self { raw: result as i64 }
    }
}

impl Div for FixedDecimal {
    type Output = Self;

    #[inline(always)]
    fn div(self, other: Self) -> Self {
        if other.is_zero() {
            panic!("Division by zero");
        }
        let result = (self.raw as i128 * Self::SCALE_FACTOR as i128) / other.raw as i128;
        Self { raw: result as i64 }
    }
}

impl fmt::Display for FixedDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let abs_raw = self.raw.abs();
        let whole = abs_raw / Self::SCALE_FACTOR;
        let frac = abs_raw % Self::SCALE_FACTOR;

        if frac == 0 {
            if self.is_negative() {
                write!(f, "-{whole}")
            } else {
                write!(f, "{whole}")
            }
        } else {
            let frac_str = format!("{frac:012}");
            let trimmed = frac_str.trim_end_matches('0');
            if self.is_negative() {
                write!(f, "-{whole}.{trimmed}")
            } else {
                write!(f, "{whole}.{trimmed}")
            }
        }
    }
}

impl FromStr for FixedDecimal {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let is_negative = s.starts_with('-');
        let s = if is_negative { &s[1..] } else { s };

        let parts: Vec<&str> = s.split('.').collect();
        let result = match parts.len() {
            1 => {
                // Whole number only
                let whole = parts[0].parse::<i64>().map_err(|_| "Invalid whole number")?;
                if whole.abs() > 1_000_000 {
                    return Err("Whole number exceeds maximum value of 1 million");
                }
                let raw = whole * Self::SCALE_FACTOR;
                Ok(Self::new(if is_negative { -raw } else { raw }))
            }
            2 => {
                // Whole and decimal parts
                let whole = parts[0].parse::<i64>().map_err(|_| "Invalid whole number")?;
                if whole.abs() > 1_000_000 {
                    return Err("Whole number exceeds maximum value of 1 million");
                }
                let decimal_str = parts[1];
                let decimal_len = decimal_str.len();
                // Pad or truncate the decimal part to match our scale
                let decimal_value = if decimal_len <= Self::SCALE as usize {
                    // Pad with zeros if needed
                    let padded = format!("{:0<12}", decimal_str);
                    padded.parse::<i64>().map_err(|_| "Invalid decimal part")?
                } else {
                    // Truncate if longer than our scale
                    let truncated = &decimal_str[..Self::SCALE as usize];
                    let padded = format!("{:0<12}", truncated);
                    padded.parse::<i64>().map_err(|_| "Invalid decimal part")?
                };

                let raw = whole * Self::SCALE_FACTOR + decimal_value;
                Ok(Self::new(if is_negative { -raw } else { raw }))
            }
            _ => Err("Invalid decimal format"),
        }?;

        Ok(result)
    }
}

impl FixedDecimal {
    #[inline(always)]
    pub fn price_levels(self, tick_size: Self) -> impl Iterator<Item = Self> {
        let raw_tick = tick_size.raw;
        let start_raw = self.raw;
        (0..).map(move |i| Self::new(start_raw + i * raw_tick))
    }

    #[inline(always)]
    pub fn round_down_to_tick(self, tick_size: Self) -> Self {
        let raw_tick = tick_size.raw;
        Self::new((self.raw / raw_tick) * raw_tick)
    }

    #[inline(always)]
    pub fn percentage_of(self, other: Self) -> Self {
        if other.is_zero() {
            Self::ZERO
        } else {
            (self * Self::from_int(100)) / other
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use crate::fixed_decimal::FixedDecimal;

    #[test]
    fn test_basic_arithmetic() {
        let a = FixedDecimal::from_parts(100, 50000000); // 100.5
        let b = FixedDecimal::from_parts(50, 25000000); // 50.25

        assert_eq!((a + b).to_string(), "150.75");
        assert_eq!((a - b).to_string(), "50.25");
        assert_eq!((a * b).to_string(), "5050.125");
        assert_eq!((a / b).to_string(), "2");
    }

    #[test]
    fn test_negative_arithmetic() {
        let a = FixedDecimal::from_str("-100.5").unwrap();
        let b = FixedDecimal::from_str("50.25").unwrap();

        assert_eq!((a + b).to_string(), "-50.25");
        assert_eq!((a - b).to_string(), "-150.75");
        assert_eq!((a * b).to_string(), "-5050.125");
        assert_eq!((a / b).to_string(), "-2");
    }

    #[test]
    fn test_from_string() {
        assert_eq!(FixedDecimal::from_str("123.45").unwrap().to_string(), "123.45");
        assert_eq!(FixedDecimal::from_str("100").unwrap().to_string(), "100");
        assert_eq!(FixedDecimal::from_str("-0.123").unwrap().to_string(), "-0.123");
        assert_eq!(FixedDecimal::from_str("-100").unwrap().to_string(), "-100");
        assert_eq!(FixedDecimal::from_str("-100.123").unwrap().to_string(), "-100.123");
    }

    #[test]
    fn test_zero_cases() {
        assert_eq!(FixedDecimal::from_str("0").unwrap().to_string(), "0");
        assert_eq!(FixedDecimal::from_str("0.0").unwrap().to_string(), "0");
        assert_eq!(FixedDecimal::from_str("-0").unwrap().to_string(), "0");
        assert_eq!(FixedDecimal::from_str("-0.0").unwrap().to_string(), "0");
    }

    #[test]
    fn test_decimal_places() {
        assert_eq!(FixedDecimal::from_str("0.12345678").unwrap().to_string(), "0.12345678");
        assert_eq!(FixedDecimal::from_str("0.1").unwrap().to_string(), "0.1");
        assert_eq!(FixedDecimal::from_str("-0.1").unwrap().to_string(), "-0.1");
    }

    #[test]
    fn test_error_cases() {
        assert!(FixedDecimal::from_str("").is_err());
        assert!(FixedDecimal::from_str(".").is_err());
        assert!(FixedDecimal::from_str("abc").is_err());
        assert!(FixedDecimal::from_str("1.2.3").is_err());
    }

    #[test]
    fn test_from_f64() {
        // Test basic conversion
        assert_eq!(FixedDecimal::from_f64(123.45).to_string(), "123.45");
        assert_eq!(FixedDecimal::from_f64(-123.45).to_string(), "-123.45");
        // Test zero
        assert_eq!(FixedDecimal::from_f64(0.0).to_string(), "0");
        assert_eq!(FixedDecimal::from_f64(-0.0).to_string(), "0");
        // Test small decimals
        assert_eq!(FixedDecimal::from_f64(0.12345678).to_string(), "0.12345678");
        assert_eq!(FixedDecimal::from_f64(-0.12345678).to_string(), "-0.12345678");
        // Test precision handling
        assert_eq!(FixedDecimal::from_f64(1.23456789).to_string(), "1.23456789");
        assert_eq!(FixedDecimal::from_f64(-1.23456789).to_string(), "-1.23456789");
        // Test special cases
        assert_eq!(FixedDecimal::from_f64(f64::NAN), FixedDecimal::ZERO);
        assert_eq!(FixedDecimal::from_f64(f64::INFINITY), FixedDecimal::MAX);
        assert_eq!(FixedDecimal::from_f64(f64::NEG_INFINITY), FixedDecimal::MIN);
        // Test very large and small numbers
        let max = (i64::MAX as f64) / FixedDecimal::SCALE_FACTOR as f64;
        let min = (i64::MIN as f64) / FixedDecimal::SCALE_FACTOR as f64;
        assert_eq!(FixedDecimal::from_f64(max), FixedDecimal::MAX);
        assert_eq!(FixedDecimal::from_f64(min), FixedDecimal::MIN);
    }

    #[test]
    fn test_f64_round_trip() {
        let test_values = [1.5, -1.5, 100.125, -100.125, 0.00000001, -0.00000001];
        for &value in &test_values {
            let decimal = FixedDecimal::from_f64(value);
            let round_trip = decimal.to_f64();
            assert!((round_trip - value).abs() < 1e-8, "Round trip failed for {}: got {}", value, round_trip);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod serde_tests {
    use std::str::FromStr as _;

    use crate::fixed_decimal::FixedDecimal;

    #[test]
    fn test_serde_json_roundtrip() {
        let original = FixedDecimal::from_str("123.456").unwrap();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: FixedDecimal = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
        assert_eq!(deserialized.to_string(), "123.456");
    }

    #[test]
    fn test_deserialize_struct() {
        #[derive(Debug, serde::Deserialize)]
        #[allow(dead_code)]
        struct Test {
            x: FixedDecimal,
            y: FixedDecimal,
        }

        let json = r#"{"x": 232124, "y": "212123.91212"}"#;
        let deserialized: Test = serde_json::from_str(json).unwrap();
        insta::assert_snapshot!(deserialized.x);
        insta::assert_snapshot!(deserialized.y);
    }

    #[test]
    fn test_deserialize_from_string() {
        let json = "\"123.456\"";
        let deserialized: FixedDecimal = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.to_string(), "123.456");
    }

    #[test]
    fn test_deserialize_from_integer() {
        let json = "123";
        let deserialized: FixedDecimal = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.to_string(), "123");
    }

    #[test]
    fn test_deserialize_from_float() {
        let json = "123.456";
        let deserialized: FixedDecimal = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.to_string(), "123.456");
    }
}
