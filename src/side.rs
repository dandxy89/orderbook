#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    #[inline(always)]
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn is_buy(self) -> bool {
        matches!(self, Self::Buy)
    }
}
