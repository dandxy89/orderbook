use crate::{decimals::decimal_type::DecimalType, event_kind::EventKind, level::Level, side::Side};

#[derive(Debug)]
pub struct Event<V: DecimalType> {
    pub kind: EventKind,
    pub side: Side,
    pub price: V,
    pub size: V,
    pub timestamp: i64,
    pub sequence_id: u64,
}

impl<V: DecimalType> Event<V> {
    #[inline(always)]
    #[must_use]
    pub const fn new(kind: EventKind, side: Side, price: V, size: V, timestamp: i64) -> Self {
        Self { kind, side, price, size, timestamp, sequence_id: 0 }
    }

    #[inline(always)]
    #[must_use]
    pub fn with_sequence_id(self, sequence_id: u64) -> Self {
        Self { sequence_id, ..self }
    }

    #[inline(always)]
    #[must_use]
    pub fn to_level(self) -> Level<V> {
        Level { price: self.price, size: self.size }
    }
}
