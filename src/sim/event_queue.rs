use super::*;
use std::collections::BinaryHeap;

#[derive(Debug, Default, Clone)]
pub struct EventQueue {
    heap: BinaryHeap<TimedEvent>,
    next_event_id: usize,
}
impl EventQueue {
    pub fn new() -> Self {
        Self {
            heap: Default::default(),
            next_event_id: 0,
        }
    }
    pub fn push(&mut self, time_due: SimSeconds, event: Event) {
        self.heap.push(TimedEvent {
            time_due,
            event,
            id: self.next_event_id,
        });
        self.next_event_id += 1;
    }
    pub fn pop(&mut self) -> Option<(SimSeconds, Event)> {
        self.heap.pop().map(|te| (te.time_due, te.event))
    }
    pub fn peek(&self) -> Option<(SimSeconds, Event)> {
        self.heap.peek().map(|te| (te.time_due, te.event))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct TimedEvent {
    time_due: SimSeconds,
    event: Event,
    id: usize,
}
impl Ord for TimedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Ordered by importance:
        // the most important event is the one with the lowest `time_due`...
        let time_based = self.time_due.cmp(&other.time_due).reverse();
        if time_based == std::cmp::Ordering::Equal {
            // if `time_due` is identical, events scheduled first are more important
            self.id.cmp(&other.id).reverse()
        } else {
            time_based
        }
    }
}
impl PartialOrd for TimedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
