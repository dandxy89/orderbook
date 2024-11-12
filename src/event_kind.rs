#[derive(Debug, PartialEq, Eq)]
pub enum EventKind {
    /// Trade events
    Trade,
    /// Best Bid/Offer events
    BBO,
    /// Level 2 events (prices and sizes)
    L2,
}
