#![allow(clippy::enum_glob_use)]
pub use hecs::{Entity, World};
pub use rand::prelude::{IteratorRandom, Rng, SliceRandom};
pub use std::error::Error;

use rand::rngs::ThreadRng;
use std::collections::VecDeque;

mod event_handler;
mod event_queue;
mod logger;
mod time;
mod underlay;

pub use event_handler::*;
pub use event_queue::{EventQueue, TimedEvent};
pub use logger::*;
pub use time::*;
pub use underlay::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SimEvent {
    ExternalCommand(SimCommand),
    MessageArrived(Entity),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SimCommand {
    SpawnRandomNodes(usize),
    SpawnRandomMessages(usize),
    StartRandomWalk(Entity, usize),
    AddRandomPeersToEachNode(usize, usize),
    MakeDelaunayNetwork,
}

pub struct Simulation {
    pub world: World,
    pub time: Time,
    pub rng: ThreadRng,
    pub logger: Logger,
    event_queue: EventQueue,
    underlay_config: UnderlayConfig,
}
impl Simulation {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            time: Time::new(0.02),
            rng: rand::thread_rng(),
            logger: Logger::new(),
            event_queue: EventQueue::new(),
            underlay_config: UnderlayConfig::new(1000., 1000.),
        }
    }
    pub fn schedule_immediate(&mut self, event: SimEvent) {
        self.schedule_at(self.time.now() + f64::EPSILON, event)
    }
    pub fn schedule_in(&mut self, duration: SimSeconds, event: SimEvent) {
        self.schedule_at(self.time.now() + duration, event)
    }
    pub fn schedule_at(&mut self, time_due: SimSeconds, event: SimEvent) {
        self.event_queue.push(TimedEvent { time_due, event });
    }
    pub fn catch_up(
        &mut self,
        event_handlers: &mut [&mut impl EventHandler],
        event_handlers_mut: &mut [&mut impl EventHandlerMut],
        elapsed_real_time: RealSeconds,
    ) {
        self.work_until(
            event_handlers,
            event_handlers_mut,
            self.time.after(elapsed_real_time),
        )
    }
    fn work_until(
        &mut self,
        event_handlers: &mut [&mut impl EventHandler],
        event_handlers_mut: &mut [&mut impl EventHandlerMut],
        sim_time: SimSeconds,
    ) {
        while self
            .event_queue
            .peek()
            .filter(|event| event.time_due <= sim_time)
            .is_some()
        {
            let TimedEvent { time_due, event } = self.event_queue.pop().unwrap();
            self.time.advance_sim_time_to(time_due);
            if let Err(e) = self.handle_event(event_handlers, event_handlers_mut, event) {
                self.log(format!("Error handling event: {}", e));
            }
        }
        self.time.advance_sim_time_to(sim_time);
    }
    fn handle_event(
        &mut self,
        event_handlers: &mut [&mut impl EventHandler],
        event_handlers_mut: &mut [&mut impl EventHandlerMut],
        event: SimEvent,
    ) -> Result<(), Box<dyn Error>> {
        for handler in event_handlers_mut.iter_mut() {
            handler.handle_event(self, event)?;
        }
        for handler in event_handlers.iter_mut() {
            handler.handle_event(self, event)?;
        }
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
    fn schedule_immediate_maintains_schedule_order() {
        let mut sim = Simulation::new();
        let event1 = SimEvent::ExternalCommand(SimCommand::SpawnRandomNodes(23));
        let event2 = SimEvent::ExternalCommand(SimCommand::MakeDelaunayNetwork);
        let event3 = SimEvent::ExternalCommand(SimCommand::SpawnRandomNodes(42));

        sim.schedule_immediate(event1);
        sim.schedule_immediate(event2);
        sim.schedule_immediate(event3);
        assert_eq!(event1, sim.event_queue.pop().unwrap().event);
        assert_eq!(event2, sim.event_queue.pop().unwrap().event);
        assert_eq!(event3, sim.event_queue.pop().unwrap().event);
    }
}
