#[macro_export]
macro_rules! fixed {
    ($val:literal i64) => {
        $crate::decimals::fixed_decimal::FixedDecimal::from_int($val)
    };
    ($val:literal i32) => {
        $crate::decimals::fixed_decimal::FixedDecimal::from_int($val as i64)
    };
    ($val:literal i16) => {
        $crate::decimals::fixed_decimal::FixedDecimal::from_int($val as i64)
    };
    ($val:literal i8) => {
        $crate::decimals::fixed_decimal::FixedDecimal::from_int($val as i64)
    };
    ($val:literal u64) => {
        $crate::decimals::fixed_decimal::FixedDecimal::from_int($val as i64)
    };
    ($val:literal u32) => {
        $crate::decimals::fixed_decimal::FixedDecimal::from_int($val as i64)
    };
    ($val:literal u16) => {
        $crate::decimals::fixed_decimal::FixedDecimal::from_int($val as i64)
    };
    ($val:literal u8) => {
        $crate::decimals::fixed_decimal::FixedDecimal::from_int($val as i64)
    };
    (-$val:literal f64) => {{
        let s = concat!("-", stringify!($val));
        $crate::decimals::fixed_decimal::FixedDecimal::from_str(s).unwrap()
    }};
    (-$val:literal f32) => {{
        let s = concat!("-", stringify!($val));
        $crate::decimals::fixed_decimal::FixedDecimal::from_str(s).unwrap()
    }};
    ($val:literal f64) => {{
        let s = stringify!($val);
        $crate::decimals::fixed_decimal::FixedDecimal::from_str(s).unwrap()
    }};
    ($val:literal f32) => {{
        let s = stringify!($val);
        $crate::decimals::fixed_decimal::FixedDecimal::from_str(s).unwrap()
    }};
    ($val:literal) => {
        $crate::decimals::fixed_decimal::FixedDecimal::from_f64($val as f64)
    };
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_fixed_macro_integer_types() {
        assert_eq!(fixed!(100i64).to_string(), "100");
        assert_eq!(fixed!(100i32).to_string(), "100");
        assert_eq!(fixed!(100i16).to_string(), "100");
        assert_eq!(fixed!(100i8).to_string(), "100");
        assert_eq!(fixed!(100u64).to_string(), "100");
        assert_eq!(fixed!(100u32).to_string(), "100");
        assert_eq!(fixed!(100u16).to_string(), "100");
        assert_eq!(fixed!(100u8).to_string(), "100");
        assert_eq!(fixed!(-100i64).to_string(), "-100");
        assert_eq!(fixed!(-100i32).to_string(), "-100");
        assert_eq!(fixed!(-100i16).to_string(), "-100");
        assert_eq!(fixed!(-100i8).to_string(), "-100");
    }

    #[test]
    fn test_fixed_macro_float_types() {
        assert_eq!(fixed!(100.5f64).to_string(), "100.5");
        assert_eq!(fixed!(100.5f32).to_string(), "100.5");
        assert_eq!(fixed!(-100.5f64).to_string(), "-100.5");
        assert_eq!(fixed!(-100.5f32).to_string(), "-100.5");
    }

    #[test]
    fn test_fixed_macro_defaults() {
        assert_eq!(fixed!(100).to_string(), "100");
        assert_eq!(fixed!(100.5).to_string(), "100.5");
        assert_eq!(fixed!(-100).to_string(), "-100");
        assert_eq!(fixed!(-100.5).to_string(), "-100.5");
    }

    #[test]
    fn test_fixed_macro_precision() {
        assert_eq!(fixed!(1.234567891234).to_string(), "1.234567891234");
        assert_eq!(fixed!(1.23456789).to_string(), "1.23456789");
        assert_eq!(fixed!(-1.23456789).to_string(), "-1.23456789");
        assert_eq!(fixed!(-1.23456789).to_string(), "-1.23456789");
    }

    #[test]
    fn test_small_fixed_macro_precision() {
        assert_eq!(fixed!(1.23).to_string(), "1.23");
        assert_eq!(fixed!(-1.23).to_string(), "-1.23");
    }
}
