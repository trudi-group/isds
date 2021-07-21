#![allow(clippy::enum_glob_use)]
use super::*;
use rand::prelude::*;
use std::collections::VecDeque;

mod event_queue;
mod generic;
mod underlay;

mod bitcoin;

pub use event_queue::{EventQueue, TimedEvent};
pub use generic::{PeerSet, TimeSpan};
pub use underlay::{UnderlayLine, UnderlayMessage, UnderlayNodeName, UnderlayPosition};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SimEvent {
    ExternalCommand(SimCommand),
    MessageArrived(Entity),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SimCommand {
    SpawnRandomNodes(u32),
    SpawnRandomMessages(u32),
    FormConnections(u32),
}

pub struct Simulator {
    event_queue: EventQueue,
    pub message_log: VecDeque<(SimSeconds, String)>,
    rng: ThreadRng,
}
impl Simulator {
    pub fn new() -> Self {
        Self {
            event_queue: EventQueue::new(),
            message_log: VecDeque::new(),
            rng: rand::thread_rng(),
        }
    }
    pub fn schedule(&mut self, time_due: SimSeconds, event: SimEvent) {
        self.event_queue.push(TimedEvent {
            time: time_due,
            event,
        });
    }
    pub fn work_until(&mut self, world: &mut World, sim_time: SimSeconds) {
        while self
            .event_queue
            .peek()
            .filter(|event| event.time <= sim_time)
            .is_some()
        {
            let timed_event = self.event_queue.pop().unwrap();
            let sim_time_now = timed_event.time;
            let event = timed_event.event;
            self.apply_event(world, sim_time_now, event);
        }
    }
    fn apply_event(&mut self, world: &mut World, sim_time_now: SimSeconds, event: SimEvent) {
        use SimCommand::*;
        use SimEvent::*;
        match event {
            MessageArrived(message_ent) => {
                let underlay_message = world.get::<UnderlayMessage>(message_ent).unwrap();
                self.log(
                    sim_time_now,
                    format!(
                        "{}: Got message from {}",
                        name(world, underlay_message.dest),
                        name(world, underlay_message.source),
                    ),
                );
                let new_source = underlay_message.dest;
                drop(underlay_message);
                world.despawn(message_ent).unwrap();
                self.spawn_message_to_random_node(world, sim_time_now, new_source)
                    .unwrap();
            }
            ExternalCommand(command) => match command {
                SpawnRandomNodes(count) => {
                    for _ in 0..count {
                        self.spawn_random_node(world);
                    }
                }
                SpawnRandomMessages(count) => {
                    for _ in 0..count {
                        self.spawn_message_between_random_nodes(world, sim_time_now)
                            .unwrap();
                    }
                }
                FormConnections(count) => {
                    for _ in 0..count {
                        self.form_directed_link_between_two_random_nodes(world)
                            .unwrap();
                    }
                }
            },
        }
    }
    fn log(&mut self, sim_time: SimSeconds, message: String) {
        self.message_log.push_front((sim_time, message));
        self.message_log.truncate(12);
    }
}
