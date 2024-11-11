use rust_decimal::Decimal;

use crate::{event_kind::EventKind, level::Level, side::Side};

#[derive(Debug)]
pub struct Event {
    pub kind: EventKind,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
    pub timestamp: i64,
    pub sequence_id: u64,
}

impl Event {
    #[inline(always)]
    #[must_use]
    pub const fn new(kind: EventKind, side: Side, price: Decimal, size: Decimal, timestamp: i64) -> Self {
        Self { kind, side, price, size, timestamp, sequence_id: 0 }
    }

    #[inline(always)]
    #[must_use]
    pub const fn with_sequence_id(self, sequence_id: u64) -> Self {
        Self { sequence_id, ..self }
    }

    #[inline(always)]
    #[must_use]
    pub const fn to_level(self) -> Level {
        Level { price: self.price, size: self.size }
    }
}
