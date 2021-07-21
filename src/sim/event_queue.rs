use super::*;
use std::collections::BinaryHeap;

pub type EventQueue = BinaryHeap<TimedEvent>;

#[derive(Debug, Eq, PartialEq)]
pub struct TimedEvent {
    pub time: SimSeconds,
    pub event: SimEvent,
}
impl Ord for TimedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Ordered by importance:
        // the most important event is the one with the lowest `time_due`...
        self.time.cmp(&other.time).reverse()
    }
}
impl PartialOrd for TimedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
