use rust_decimal::Decimal;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct Level {
    pub price: Decimal,
    pub size: Decimal,
}

impl Level {
    #[inline(always)]
    #[must_use]
    pub const fn new(price: Decimal, size: Decimal) -> Self {
        Self { price, size }
    }

    #[inline(always)]
    #[must_use]
    pub const fn bound(is_min: bool) -> Self {
        Self { price: if is_min { Decimal::MIN } else { Decimal::MAX }, size: Decimal::ZERO }
    }

    #[inline(always)]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.price > Decimal::ZERO
    }
}
