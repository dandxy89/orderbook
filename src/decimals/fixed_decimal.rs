use std::{
    fmt,
    iter::Sum,
    mem::transmute,
    ops::{Add, Div, Mul, Rem, Sub, SubAssign},
    str::FromStr,
};

use crate::decimals::decimal_type::DecimalType;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedDecimal {
    raw: i64,
}

// Constants for bit manipulation
impl FixedDecimal {
    const SCALE: i32 = 13;
    const SCALE_FACTOR: i64 = 10_000_000_000_000;
    const SIGN_MASK: i64 = 1 << 63;
    const VALUE_MASK: i64 = !Self::SIGN_MASK;

    pub const ZERO: Self = Self { raw: 0 };
    pub const ONE: Self = Self { raw: Self::SCALE_FACTOR };
    pub const TWO: Self = Self { raw: 2 * Self::SCALE_FACTOR };
    pub const TEN: Self = Self { raw: 10 * Self::SCALE_FACTOR };
    pub const MAX: Self = Self { raw: i64::MAX };
    pub const MIN: Self = Self { raw: i64::MIN };
    pub const ONE_HUNDRED: Self = Self { raw: 100 * Self::SCALE_FACTOR };
    pub const ONE_THOUSAND: Self = Self { raw: 1_000 * Self::SCALE_FACTOR };

    const POW10_TABLE: [i64; 19] = [
        1,
        10,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
        10_000_000_000,
        100_000_000_000,
        1_000_000_000_000,
        10_000_000_000_000,
        100_000_000_000_000,
        1_000_000_000_000_000,
        10_000_000_000_000_000,
        100_000_000_000_000_000,
        1_000_000_000_000_000_000,
    ];
}

// Safety assertions
const _: () = assert!(FixedDecimal::SCALE > 0, "Scale must be positive");
const _: () = assert!(FixedDecimal::SCALE <= 18, "Scale too large for i64");
const _: () = assert!(FixedDecimal::SCALE_FACTOR == 10_i64.pow(FixedDecimal::SCALE as u32), "Scale factor must match scale");
const _: () = assert!(FixedDecimal::SCALE_FACTOR <= i64::MAX / 1000, "Scale factor too large for safe multiplication");

impl DecimalType for FixedDecimal {
    const ZERO: Self = Self::ZERO;
    const ONE: Self = Self::ONE;
    const TWO: Self = Self::TWO;
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const ONE_HUNDRED: Self = Self::ONE_HUNDRED;
}

impl FixedDecimal {
    #[inline(always)]
    pub const fn new(raw: i64) -> Self {
        Self { raw }
    }

    #[inline(always)]
    pub const fn abs(self) -> Self {
        let mask = self.raw >> 63;
        Self { raw: (self.raw + mask) ^ mask }
    }

    #[inline(always)]
    pub const fn is_negative(self) -> bool {
        (self.raw & Self::SIGN_MASK) != 0
    }

    #[inline(always)]
    pub const fn is_zero(self) -> bool {
        (self.raw & Self::VALUE_MASK) == 0
    }

    #[inline(always)]
    pub const fn raw_value(self) -> i64 {
        self.raw
    }

    #[inline(always)]
    pub fn from_parts(whole: i64, decimal: u32) -> Self {
        // Handle special case for zero
        if whole == 0 && decimal == 0 {
            return Self::ZERO;
        }

        // Convert decimal part to proper fraction
        let decimal_str = decimal.to_string();
        let decimal_len = decimal_str.len();

        // Calculate the scaling needed
        let decimal_value = if decimal_len <= 8 {
            let mut padded = decimal_str;
            padded.push_str(&"0".repeat(8 - decimal_len));
            let decimal_val = padded.parse::<i64>().unwrap_or(0);
            decimal_val * 10_i64.pow(Self::SCALE as u32 - 8)
        } else {
            let truncated = &decimal_str[..8];
            let decimal_val = truncated.parse::<i64>().unwrap_or(0);
            decimal_val * 10_i64.pow(Self::SCALE as u32 - 8)
        };

        let whole_part = match whole.checked_mul(Self::SCALE_FACTOR) {
            Some(w) => w,
            None => return if whole < 0 { Self::MIN } else { Self::MAX },
        };

        Self { raw: whole_part.saturating_add(decimal_value) }
    }

    #[inline(always)]
    pub const fn from_int(value: i64) -> Self {
        if value.abs() > Self::SCALE_FACTOR {
            Self { raw: value }
        } else {
            Self { raw: value * Self::SCALE_FACTOR }
        }
    }

    #[inline(always)]
    pub fn from_usize(value: usize) -> Self {
        Self::from_int(value as i64)
    }

    #[inline(always)]
    pub fn from_f64(value: f64) -> Self {
        let bits: u64 = unsafe { transmute(value) };
        let exp = ((bits >> 52) & 0x7FF) as i32 - 1023;

        if exp == 1024 {
            if bits & 0xFFFFFFFFFFFFF != 0 {
                return Self::ZERO;
            }
            return if bits & (1 << 63) != 0 { Self::MIN } else { Self::MAX };
        }

        let scaled = value * Self::SCALE_FACTOR as f64;
        if scaled >= i64::MAX as f64 {
            return Self::MAX;
        }
        if scaled <= i64::MIN as f64 {
            return Self::MIN;
        }

        Self { raw: scaled.round() as i64 }
    }

    #[inline(always)]
    pub fn to_f64(self) -> f64 {
        (self.raw as f64) / (Self::SCALE_FACTOR as f64)
    }

    #[inline(always)]
    pub fn rescale(&mut self, scale: u32) {
        if scale >= Self::SCALE as u32 {
            return;
        }

        let scale_diff = Self::SCALE as u32 - scale;
        let divisor = Self::power_of_ten(scale_diff);
        self.raw = (self.raw / divisor) * divisor;
    }

    #[inline(always)]
    pub fn with_exponent(value: i64, exponent: i32) -> Self {
        let adjustment = Self::SCALE + exponent;

        // Use bit manipulation for fast zero check
        if value == 0 {
            return Self::ZERO;
        }

        // Fast path for exact scale match
        if adjustment == 0 {
            return Self { raw: value };
        }

        let abs_value = value & Self::VALUE_MASK;
        let is_negative = (value & Self::SIGN_MASK) != 0;

        if exponent < 0 {
            let scale = (-exponent) as u32;
            // Use bit shifts for powers of 10 when possible
            let divided = if scale <= 20 {
                // For small scales, use lookup table or direct division
                abs_value / Self::power_of_ten(scale)
            } else {
                // For larger scales, fallback to regular division
                abs_value / 10_i64.pow(scale)
            };

            if adjustment >= 0 {
                let result = if adjustment == 0 {
                    divided
                } else {
                    match divided.checked_mul(Self::power_of_ten(adjustment as u32)) {
                        Some(r) => r,
                        None => return if is_negative { Self::MIN } else { Self::MAX },
                    }
                };
                Self { raw: if is_negative { -result } else { result } }
            } else {
                let result = divided / Self::power_of_ten((-adjustment) as u32);
                Self { raw: if is_negative { -result } else { result } }
            }
        } else {
            if adjustment == 0 {
                return Self { raw: value };
            }
            if adjustment > 0 {
                match abs_value.checked_mul(Self::power_of_ten(adjustment as u32)) {
                    Some(result) => Self { raw: if is_negative { -result } else { result } },
                    None => {
                        if is_negative {
                            Self::MIN
                        } else {
                            Self::MAX
                        }
                    }
                }
            } else {
                let result = abs_value / Self::power_of_ten((-adjustment) as u32);
                Self { raw: if is_negative { -result } else { result } }
            }
        }
    }

    #[inline(always)]
    const fn power_of_ten(n: u32) -> i64 {
        if n < 19 {
            Self::POW10_TABLE[n as usize]
        } else {
            i64::MAX
        }
    }

    #[inline(always)]
    pub fn min(self, other: Self) -> Self {
        Self { raw: self.raw.min(other.raw) }
    }

    #[inline(always)]
    pub fn max(self, other: Self) -> Self {
        Self { raw: self.raw.max(other.raw) }
    }
}

impl Add for FixedDecimal {
    type Output = Self;

    #[inline(always)]
    fn add(self, other: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let a = std::arch::x86_64::_mm_set_epi64x(0, self.raw);
            let b = std::arch::x86_64::_mm_set_epi64x(0, other.raw);
            let sum = std::arch::x86_64::_mm_add_epi64(a, b);
            Self { raw: std::arch::x86_64::_mm_cvtsi128_si64(sum) }
        }
        #[cfg(not(target_arch = "x86_64"))]
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

impl SubAssign for FixedDecimal {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        self.raw = self.raw.saturating_sub(other.raw);
    }
}

impl Mul for FixedDecimal {
    type Output = Self;

    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::ZERO;
        }
        if self.raw == Self::SCALE_FACTOR {
            return other;
        }
        if other.raw == Self::SCALE_FACTOR {
            return self;
        }

        let a = self.raw as i128;
        let b = other.raw as i128;
        let result = (a * b) / (Self::SCALE_FACTOR as i128);

        if result > i64::MAX as i128 {
            Self::MAX
        } else if result < i64::MIN as i128 {
            Self::MIN
        } else {
            Self { raw: result as i64 }
        }
    }
}

impl Div for FixedDecimal {
    type Output = Self;

    #[inline(always)]
    fn div(self, other: Self) -> Self {
        if other.is_zero() {
            panic!("Division by zero");
        }
        if self.is_zero() {
            return Self::ZERO;
        }
        if other.raw == Self::SCALE_FACTOR {
            return self;
        }

        let a = (self.raw as i128) * (Self::SCALE_FACTOR as i128);
        let b = other.raw as i128;
        let result = a / b;

        if result > i64::MAX as i128 {
            Self::MAX
        } else if result < i64::MIN as i128 {
            Self::MIN
        } else {
            Self { raw: result as i64 }
        }
    }
}

impl Rem for FixedDecimal {
    type Output = Self;

    #[inline(always)]
    fn rem(self, other: Self) -> Self {
        if other.is_zero() {
            panic!("Division by zero");
        }
        Self { raw: self.raw % other.raw }
    }
}

impl Sum for FixedDecimal {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
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
            let frac_str = format!("{frac:013}");
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
        match parts.len() {
            1 => {
                let whole = parts[0].parse::<i64>().map_err(|_| "Invalid whole number")?;
                let raw = whole * Self::SCALE_FACTOR;
                Ok(Self { raw: if is_negative { -raw } else { raw } })
            }
            2 => {
                let whole = parts[0].parse::<i64>().map_err(|_| "Invalid whole number")?;
                let decimal_str = parts[1];
                let decimal_len = decimal_str.len();

                let decimal_value = if decimal_len <= Self::SCALE as usize {
                    let padded = format!("{:0<13}", decimal_str);
                    padded.parse::<i64>().map_err(|_| "Invalid decimal part")?
                } else {
                    let truncated = &decimal_str[..Self::SCALE as usize];
                    let padded = format!("{:0<13}", truncated);
                    padded.parse::<i64>().map_err(|_| "Invalid decimal part")?
                };

                let raw = whole * Self::SCALE_FACTOR + decimal_value;
                Ok(Self { raw: if is_negative { -raw } else { raw } })
            }
            _ => Err("Invalid decimal format"),
        }
    }
}

impl Default for FixedDecimal {
    fn default() -> Self {
        Self::ZERO
    }
}

// #[repr(transparent)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub struct FixedDecimal {
//     raw: i64,
// }
//
// impl DecimalType for FixedDecimal {
//     const ZERO: Self = Self::ZERO;
//     const ONE: Self = Self::ONE;
//     const TWO: Self = Self::TWO;
//     const MAX: Self = Self::MAX;
//     const MIN: Self = Self::MIN;
//     const ONE_HUNDRED: Self = Self::ONE_HUNDRED;
// }
//
// const _: () = assert!(FixedDecimal::SCALE > 0, "Scale must be positive");
// const _: () = assert!(FixedDecimal::SCALE <= 18, "Scale too large for i64");
// const _: () = assert!(FixedDecimal::SCALE_FACTOR == 10_i64.pow(FixedDecimal::SCALE as u32), "Scale factor must match scale");
// const _: () = assert!(FixedDecimal::SCALE_FACTOR <= i64::MAX / 1000, "Scale factor too large for safe multiplication");
//
// impl FixedDecimal {
//     const SCALE: i32 = 13;
//     const SCALE_FACTOR: i64 = 10_000_000_000_000;
//
//     pub const ZERO: Self = Self { raw: 0 };
//     pub const ONE: Self = Self { raw: Self::SCALE_FACTOR };
//     pub const TWO: Self = Self { raw: 2 * Self::SCALE_FACTOR };
//     pub const TEN: Self = Self { raw: 10 * Self::SCALE_FACTOR };
//     pub const MAX: Self = Self { raw: i64::MAX };
//     pub const MIN: Self = Self { raw: i64::MIN };
//     pub const ONE_HUNDRED: Self = Self { raw: 100 * Self::SCALE_FACTOR };
//     pub const ONE_THOUSAND: Self = Self { raw: 1_000 * Self::SCALE_FACTOR };
//
//     pub const fn new(raw: i64) -> Self {
//         Self { raw }
//     }
//
//     pub fn abs(self) -> Self {
//         if self.raw == i64::MIN {
//             return Self::MAX;
//         }
//         Self { raw: self.raw.abs() }
//     }
//
//     pub const fn from_int(value: i64) -> Self {
//         if value.abs() > Self::SCALE_FACTOR {
//             Self { raw: value }
//         } else {
//             Self { raw: value * Self::SCALE_FACTOR }
//         }
//     }
//
//     pub fn from_usize(value: usize) -> Self {
//         Self::from_int(value as i64)
//     }
//
//     #[inline]
//     pub fn from_parts(whole: i64, decimal: u32) -> Self {
//         let decimal_digits = decimal.checked_ilog10().map_or(0, |x| x as u32 + 1);
//
//         let decimal_value = if decimal_digits <= Self::SCALE as u32 {
//             (decimal as i64) * 10_i64.pow(Self::SCALE as u32 - decimal_digits)
//         } else {
//             (decimal as i64) / 10_i64.pow(decimal_digits - Self::SCALE as u32)
//         };
//
//         let raw = whole.checked_mul(Self::SCALE_FACTOR).and_then(|x| x.checked_add(decimal_value)).unwrap_or_else(|| {
//             if whole.is_negative() {
//                 i64::MIN
//             } else {
//                 i64::MAX
//             }
//         });
//
//         Self { raw }
//     }
//
//     pub fn to_f64(self) -> f64 {
//         self.raw as f64 / Self::SCALE_FACTOR as f64
//     }
//
//     pub const fn raw_value(self) -> i64 {
//         self.raw
//     }
//
//     pub const fn is_zero(self) -> bool {
//         self.raw == 0
//     }
//
//     pub const fn is_negative(self) -> bool {
//         self.raw < 0
//     }
//
//     #[cold]
//     fn handle_overflow_positive() -> Self {
//         Self::MAX
//     }
//
//     #[cold]
//     fn handle_overflow_negative() -> Self {
//         Self::MIN
//     }
//
//     pub fn with_exponent(value: i64, exponent: i32) -> Self {
//         let adjustment = Self::SCALE + exponent;
//
//         if exponent < 0 {
//             let scale = (-exponent) as u32;
//             let divided = value / 10_i64.pow(scale);
//
//             if adjustment >= 0 {
//                 if adjustment == 0 {
//                     return Self { raw: divided };
//                 }
//                 match divided.checked_mul(10_i64.pow(adjustment as u32)) {
//                     Some(result) => Self { raw: result },
//                     None => {
//                         if divided.is_negative() {
//                             Self::MIN
//                         } else {
//                             Self::MAX
//                         }
//                     }
//                 }
//             } else {
//                 Self { raw: divided / 10_i64.pow((-adjustment) as u32) }
//             }
//         } else {
//             if adjustment == 0 {
//                 return Self { raw: value };
//             }
//             if adjustment > 0 {
//                 match value.checked_mul(10_i64.pow(adjustment as u32)) {
//                     Some(result) => Self { raw: result },
//                     None => {
//                         if value.is_negative() {
//                             Self::MIN
//                         } else {
//                             Self::MAX
//                         }
//                     }
//                 }
//             } else {
//                 Self { raw: value / 10_i64.pow((-adjustment) as u32) }
//             }
//         }
//     }
//
//     pub fn rescale(&mut self, scale: u32) {
//         if scale >= Self::SCALE as u32 {
//             return;
//         }
//
//         let scale_diff = Self::SCALE as u32 - scale;
//         let divisor = 10_i64.pow(scale_diff);
//         self.raw = (self.raw / divisor) * divisor;
//     }
//
//     pub fn from_f64(value: f64) -> Self {
//         if !value.is_finite() {
//             return if value.is_nan() {
//                 Self::ZERO
//             } else if value.is_sign_positive() {
//                 Self::MAX
//             } else {
//                 Self::MIN
//             };
//         }
//
//         let scaled = value * Self::SCALE_FACTOR as f64;
//         if scaled >= i64::MAX as f64 {
//             return Self::MAX;
//         }
//         if scaled <= i64::MIN as f64 {
//             return Self::MIN;
//         }
//
//         Self { raw: scaled.round() as i64 }
//     }
//
//     #[inline(always)]
//     pub fn min(self, other: Self) -> Self {
//         Self { raw: self.raw.min(other.raw) }
//     }
//
//     #[inline(always)]
//     pub fn max(self, other: Self) -> Self {
//         Self { raw: self.raw.max(other.raw) }
//     }
// }
//
// impl Default for FixedDecimal {
//     fn default() -> Self {
//         Self::ZERO
//     }
// }
//
// impl Add for FixedDecimal {
//     type Output = Self;
//
//     fn add(self, other: Self) -> Self {
//         Self { raw: self.raw.saturating_add(other.raw) }
//     }
// }
//
// impl Sub for FixedDecimal {
//     type Output = Self;
//
//     fn sub(self, other: Self) -> Self {
//         Self { raw: self.raw.saturating_sub(other.raw) }
//     }
// }
//
// impl SubAssign for FixedDecimal {
//     fn sub_assign(&mut self, other: Self) {
//         self.raw = self.raw.saturating_sub(other.raw);
//     }
// }
//
// impl Mul for FixedDecimal {
//     type Output = Self;
//
//     #[inline]
//     fn mul(self, other: Self) -> Self {
//         if self.is_zero() || other.is_zero() {
//             return Self::ZERO;
//         }
//
//         let result = (self.raw as i128 * other.raw as i128) / Self::SCALE_FACTOR as i128;
//         if result > i64::MAX as i128 {
//             return Self::handle_overflow_positive();
//         }
//         if result < i64::MIN as i128 {
//             return Self::handle_overflow_negative();
//         }
//
//         Self { raw: result as i64 }
//     }
// }
//
// impl Div for FixedDecimal {
//     type Output = Self;
//
//     #[inline]
//     fn div(self, other: Self) -> Self {
//         if other.is_zero() {
//             panic!("Division by zero");
//         }
//
//         if self.is_zero() {
//             return Self::ZERO;
//         }
//
//         let scaled_dividend = (self.raw as i128) * Self::SCALE_FACTOR as i128;
//         let result = scaled_dividend / other.raw as i128;
//
//         if result > i64::MAX as i128 {
//             return Self::handle_overflow_positive();
//         }
//         if result < i64::MIN as i128 {
//             return Self::handle_overflow_negative();
//         }
//
//         Self { raw: result as i64 }
//     }
// }
//
// impl Rem for FixedDecimal {
//     type Output = Self;
//
//     fn rem(self, other: Self) -> Self {
//         if other.is_zero() {
//             panic!("Division by zero");
//         }
//
//         Self { raw: self.raw % other.raw }
//     }
// }
//
// impl Sum for FixedDecimal {
//     fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
//         iter.fold(Self::ZERO, Add::add)
//     }
// }
//
// impl fmt::Display for FixedDecimal {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let abs_raw = self.raw.abs();
//         let whole = abs_raw / Self::SCALE_FACTOR;
//         let frac = abs_raw % Self::SCALE_FACTOR;
//
//         if frac == 0 {
//             if self.is_negative() {
//                 write!(f, "-{whole}")
//             } else {
//                 write!(f, "{whole}")
//             }
//         } else {
//             let frac_str = format!("{:013}", frac);
//             let trimmed = frac_str.trim_end_matches('0');
//             if self.is_negative() {
//                 write!(f, "-{whole}.{trimmed}")
//             } else {
//                 write!(f, "{whole}.{trimmed}")
//             }
//         }
//     }
// }
//
// impl FromStr for FixedDecimal {
//     type Err = &'static str;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let is_negative = s.starts_with('-');
//         let s = if is_negative { &s[1..] } else { s };
//
//         let parts: Vec<&str> = s.split('.').collect();
//         match parts.len() {
//             1 => {
//                 // Whole number only
//                 let whole = parts[0].parse::<i64>().map_err(|_| "Invalid whole number")?;
//                 let raw = whole * Self::SCALE_FACTOR;
//                 Ok(Self { raw: if is_negative { -raw } else { raw } })
//             }
//             2 => {
//                 // Whole and decimal parts
//                 let whole = parts[0].parse::<i64>().map_err(|_| "Invalid whole number")?;
//                 let decimal_str = parts[1];
//                 let decimal_len = decimal_str.len();
//
//                 // Pad or truncate the decimal part to match our scale
//                 let decimal_value = if decimal_len <= 13 {
//                     // Pad with zeros if needed
//                     let padded = format!("{:0<13}", decimal_str);
//                     padded.parse::<i64>().map_err(|_| "Invalid decimal part")?
//                 } else {
//                     // Truncate if longer than our scale
//                     let truncated = &decimal_str[..13];
//                     let padded = format!("{:0<13}", truncated);
//                     padded.parse::<i64>().map_err(|_| "Invalid decimal part")?
//                 };
//
//                 let raw = whole * Self::SCALE_FACTOR + decimal_value;
//                 Ok(Self { raw: if is_negative { -raw } else { raw } })
//             }
//             _ => Err("Invalid decimal format"),
//         }
//     }
// }

#[cfg(feature = "serde")]
impl serde::Serialize for FixedDecimal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
struct FixedDecimalVisitor;

#[cfg(feature = "serde")]
#[allow(clippy::needless_lifetimes)]
impl<'de> serde::de::Visitor<'de> for FixedDecimalVisitor {
    type Value = FixedDecimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a decimal number as a string or integer")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FixedDecimal::from_int(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value <= i64::MAX as u64 {
            Ok(FixedDecimal::from_int(value as i64))
        } else {
            Err(E::custom("integer too large"))
        }
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FixedDecimal::from_f64(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        FixedDecimal::from_str(value).map_err(E::custom)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FixedDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(FixedDecimalVisitor)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use crate::decimals::fixed_decimal::FixedDecimal;

    #[test]
    fn test_basic_remainder() {
        let a = FixedDecimal::from_str("10.5").unwrap();
        let b = FixedDecimal::from_str("3.0").unwrap();
        insta::assert_debug_snapshot!(a % b);
    }

    #[test]
    fn test_negative_remainder() {
        let a = FixedDecimal::from_str("-10.5").unwrap();
        let b = FixedDecimal::from_str("3.0").unwrap();
        insta::assert_debug_snapshot!(a % b);

        let a = FixedDecimal::from_str("10.5").unwrap();
        let b = FixedDecimal::from_str("-3.0").unwrap();
        insta::assert_debug_snapshot!(a % b);

        let a = FixedDecimal::from_str("-10.5").unwrap();
        let b = FixedDecimal::from_str("-3.0").unwrap();
        insta::assert_debug_snapshot!(a % b);
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_remainder_by_zero() {
        let a = FixedDecimal::from_str("10.5").unwrap();
        let b = FixedDecimal::ZERO;
        let _result = a % b;
    }

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
        assert_eq!(FixedDecimal::from_f64(max), FixedDecimal::MAX);
        let min = (i64::MIN as f64) / FixedDecimal::SCALE_FACTOR as f64;
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

    #[test]
    fn test_rescale_basic() {
        let mut num = FixedDecimal::from_str("123.456789").unwrap();
        num.rescale(2);
        assert_eq!(num.to_string(), "123.45");
    }

    #[test]
    fn test_rescale_negative() {
        let mut num = FixedDecimal::from_str("-123.456789").unwrap();
        num.rescale(2);
        assert_eq!(num.to_string(), "-123.45");
    }

    #[test]
    fn test_rescale_higher_scale() {
        // Test no change when trying to scale beyond max precision
        let mut num = FixedDecimal::from_str("123.456789").unwrap();
        let original = num;
        num.rescale(13);
        assert_eq!(num, original);
    }

    #[test]
    fn test_rescale_multiple_times() {
        let mut num = FixedDecimal::from_str("123.456789").unwrap();
        num.rescale(4);
        assert_eq!(num.to_string(), "123.4567");
        num.rescale(2);
        assert_eq!(num.to_string(), "123.45");
    }

    #[test]
    fn test_min_notional() {
        let total_value = FixedDecimal::with_exponent(500000000, -8);
        insta::assert_debug_snapshot!(total_value);
    }

    #[test]
    fn test_abs() {
        let num = FixedDecimal::from_str("-123.456789").unwrap();
        assert_eq!(num.abs().to_string(), "123.456789");
    }
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod serde_tests {
    use std::str::FromStr as _;

    use crate::decimals::fixed_decimal::FixedDecimal;

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
