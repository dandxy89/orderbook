pub trait DecimalType {
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const MAX: Self;
    const MIN: Self;
    const ONE_HUNDRED: Self;
}

#[cfg(feature = "rust_decimal")]
impl DecimalType for rust_decimal::Decimal {
    const ZERO: Self = rust_decimal::Decimal::ZERO;
    const ONE: Self = rust_decimal::Decimal::ONE;
    const TWO: Self = rust_decimal::Decimal::TWO;

    const MAX: Self = rust_decimal::Decimal::MAX;
    const MIN: Self = rust_decimal::Decimal::MIN;
    const ONE_HUNDRED: Self = rust_decimal::Decimal::ONE_HUNDRED;
}
