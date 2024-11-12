use crate::decimals::decimal_type::DecimalType;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct Level<V: DecimalType> {
    pub price: V,
    pub size: V,
}

impl<V: DecimalType + PartialOrd> Level<V> {
    #[inline(always)]
    #[must_use]
    pub const fn new(price: V, size: V) -> Self {
        Self { price, size }
    }

    #[inline(always)]
    #[must_use]
    pub const fn bound(is_min: bool) -> Self {
        Self { price: if is_min { V::MIN } else { V::MAX }, size: V::ZERO }
    }

    #[inline(always)]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.price > V::ZERO
    }
}
