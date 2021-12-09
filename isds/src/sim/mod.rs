#![allow(clippy::enum_glob_use)]

#![macro_use]
extern crate gloo;
use gloo::console::log;

pub use hecs::{Entity, World};
pub use rand::prelude::{IteratorRandom, Rng, SliceRandom};
pub use std::error::Error;

use rand::rngs::ThreadRng;
use std::collections::VecDeque;
use std::mem;

mod command;
mod despawner;
mod event_handlers;
mod event_queue;
mod logger;
mod peers;
mod pokes;
mod protocol;
mod time;
mod underlay;

use despawner::Despawner;

pub use command::Command;
pub use event_handlers::{EventHandler, EventWatcher};
pub use event_queue::EventQueue;
pub use logger::Logger;
pub use protocol::{InvokeProtocolForAllNodes, NodeInterface, Payload, Protocol};
pub use time::{OrderedFloat, RealSeconds, SimSeconds, Time, TimeSpan};

pub use peers::*;
pub use pokes::*;
pub use underlay::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Event {
    Command(Entity),
    Node(Entity, NodeEvent),
    Generic(Entity),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NodeEvent {
    MessageArrived(Entity),
    TimerFired(Entity),
    PeerSetChanged(PeerSetUpdate),
    Poke,
}

pub struct Simulation {
    pub world: World,
    pub time: Time,
    pub rng: ThreadRng,
    pub logger: Logger,
    additional_event_handlers: Vec<Box<dyn EventHandler>>,
    event_queue: EventQueue,
    underlay_config: UnderlayConfig,
}
impl Simulation {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            time: Time::new(0.1),
            rng: rand::thread_rng(),
            logger: Logger::new(),
            additional_event_handlers: vec![],
            event_queue: EventQueue::new(),
            underlay_config: UnderlayConfig::new(800., 800.),
        }
    }
    pub fn add_event_handler(&mut self, event_handler: impl EventHandler + 'static) {
        self.additional_event_handlers.push(Box::new(event_handler))
    }
    pub fn schedule_now(&mut self, event: Event) {
        self.schedule_at(self.time.now(), event)
    }
    pub fn schedule_in(&mut self, duration: SimSeconds, event: Event) {
        self.schedule_at(self.time.now() + duration, event)
    }
    pub fn schedule_at(&mut self, time_due: SimSeconds, event: Event) {
        self.event_queue.push(time_due, event);
    }
    pub fn catch_up(&mut self, elapsed_real_time: RealSeconds) {
        self.catch_up_with_watchers(&mut [], elapsed_real_time)
    }
    pub fn catch_up_with_watchers(
        &mut self,
        event_watchers: &mut [&mut dyn EventWatcher],
        elapsed_real_time: RealSeconds,
    ) {
        self.work_until(event_watchers, self.time.after(elapsed_real_time))
    }
    fn work_until(&mut self, event_watchers: &mut [&mut dyn EventWatcher], sim_time: SimSeconds) {
        while self
            .event_queue
            .peek()
            .filter(|&(time_due, _)| time_due <= sim_time)
            .is_some()
        {
            let (time_due, event) = self.event_queue.pop().unwrap();
            self.time.advance_sim_time_to(time_due);
            if let Err(e) = self.handle_event(event_watchers, event) {
                self.log(format!("Error handling event: {}", e));
            }
        }
        self.time.advance_sim_time_to(sim_time);
    }
    fn handle_event(
        &mut self,
        event_watchers: &mut [&mut dyn EventWatcher],
        event: Event,
    ) -> Result<(), Box<dyn Error>> {
        command::Handler.handle_event(self, event)?;

        // TODO clean this up by splitting `Simulation` into something like `Logic` and `State`
        let mut additional_event_handlers = mem::take(&mut self.additional_event_handlers);
        for handler in additional_event_handlers.iter_mut() {
            handler.handle_event(self, event)?;
        }
        let new_handlers = mem::replace(
            &mut self.additional_event_handlers,
            additional_event_handlers,
        );
        debug_assert!(new_handlers.is_empty());

        for handler in event_watchers.iter_mut() {
            handler.handle_event(self, event)?;
        }
        Despawner.handle_event(self, event)?;
        Ok(())
    }
    pub fn name(&self, entity: Entity) -> String {
        if let Ok(entity_ref) = self.world.entity(entity) {
            if let Some(node_name) = entity_ref.get::<UnderlayNodeName>() {
                node_name.0.to_string()
            } else {
                format!("UNNAMEABLE ({})", entity.id())
            }
        } else {
            format!("INEXISTING ({})", entity.id())
        }
    }
    pub fn log(&mut self, message: String) {
        self.logger.log(self.time.now(), message);
    }
}
impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn simultaneous_events_are_executed_in_order_of_scheduling() {
        let mut sim = Simulation::new();

        let node = sim.spawn_random_node();

        let event4 = Event::Node(node, NodeEvent::Poke);
        let event5 = Event::Node(
            node,
            NodeEvent::MessageArrived(sim.world.spawn(("fake message", 73))),
        );
        let event6 = Event::Generic(sim.world.spawn((42,)));

        let target_time = OrderedFloat(120.);
        sim.schedule_at(target_time, event4);
        sim.schedule_at(target_time, event5);
        sim.schedule_at(target_time, event6);

        let event1 = Event::Generic(sim.world.spawn((23,)));
        let event2 = Event::Generic(sim.world.spawn((17,)));
        let event3 = Event::Generic(sim.world.spawn((42,)));

        sim.schedule_now(event1);
        sim.schedule_now(event2);
        sim.schedule_now(event3);

        assert_eq!(event1, sim.event_queue.pop().unwrap().1);
        assert_eq!(event2, sim.event_queue.pop().unwrap().1);
        assert_eq!(event3, sim.event_queue.pop().unwrap().1);
        assert_eq!(event4, sim.event_queue.pop().unwrap().1);
        assert_eq!(event5, sim.event_queue.pop().unwrap().1);
        assert_eq!(event6, sim.event_queue.pop().unwrap().1);
    }
}