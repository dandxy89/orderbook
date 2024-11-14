use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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

impl AsRef<str> for Side {
    fn as_ref(&self) -> &str {
        match self {
            Self::Buy => "Buy",
            Self::Sell => "Sell",
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buy => write!(f, "Buy"),
            Self::Sell => write!(f, "Sell"),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Side {
    #[inline]
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SideVisitor;

        #[allow(clippy::missing_trait_methods)]
        impl serde::de::Visitor<'_> for SideVisitor {
            type Value = Side;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing 'Buy' or 'Sell', or a number 0 or 1")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v {
                    "buy" | "BUY" | "Buy" | "0" => Ok(Side::Buy),
                    "sell" | "SELL" | "Sell" | "1" => Ok(Side::Sell),
                    _ => Err(E::unknown_field(v, &["Buy", "Sell"])),
                }
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                match v {
                    0 => Ok(Side::Buy),
                    1 => Ok(Side::Sell),
                    _ => Err(E::invalid_value(serde::de::Unexpected::Signed(v), &"0 for Buy, 1 for Sell")),
                }
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                match v {
                    0 => Ok(Side::Buy),
                    1 => Ok(Side::Sell),
                    _ => Err(E::invalid_value(serde::de::Unexpected::Unsigned(v), &"0 for Buy, 1 for Sell")),
                }
            }
        }

        deserializer.deserialize_any(SideVisitor)
    }
}
