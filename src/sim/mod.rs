#![allow(clippy::enum_glob_use)]
use super::*;
use rand::prelude::*;

mod event_queue;
mod generic;
mod underlay;

mod bitcoin;
mod random_walks;

pub use event_queue::{EventQueue, TimedEvent};
pub use generic::*;
pub use underlay::*;

use random_walks::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SimEvent {
    ExternalCommand(SimCommand),
    MessageArrived(Entity),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SimCommand {
    SpawnRandomNodes(usize),
    SpawnRandomMessages(usize),
    AddRandomPeersToEachNode(usize, usize),
    MakeDelaunayNetwork,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WorldChanges {
    pub topology: bool,
    pub new_messages: bool,
}
impl WorldChanges {
    pub fn none() -> Self {
        Self {
            topology: false,
            new_messages: false,
        }
    }
    pub fn topology() -> Self {
        Self {
            topology: true,
            ..Self::none()
        }
    }
    pub fn new_messages() -> Self {
        Self {
            new_messages: true,
            ..Self::none()
        }
    }
    pub fn update(&mut self, other: Self) {
        self.topology |= other.topology;
        self.new_messages |= other.new_messages;
    }
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
    pub fn work_until(&mut self, world: &mut World, sim_time: SimSeconds) -> WorldChanges {
        let mut changes = WorldChanges::none();
        while self
            .event_queue
            .peek()
            .filter(|event| event.time <= sim_time)
            .is_some()
        {
            let timed_event = self.event_queue.pop().unwrap();
            let sim_time_now = timed_event.time;
            let event = timed_event.event;
            changes.update(self.apply_event(world, sim_time_now, event));
        }
        changes
    }
    fn apply_event(
        &mut self,
        world: &mut World,
        sim_time_now: SimSeconds,
        event: SimEvent,
    ) -> WorldChanges {
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
                self.send_message_to_random_peer(world, sim_time_now, new_source)
                    .unwrap();
                WorldChanges::new_messages()
            }
            ExternalCommand(command) => match command {
                SpawnRandomNodes(count) => {
                    for _ in 0..count {
                        self.spawn_random_node(world);
                    }
                    WorldChanges::topology()
                }
                SpawnRandomMessages(count) => {
                    for _ in 0..count {
                        let node = pick_random_node(world, &mut self.rng).unwrap();
                        self.send_message_to_random_peer(world, sim_time_now, node)
                            .unwrap();
                        // self.send_message_to_random_node(world, sim_time_now, node)
                        //     .unwrap();
                    }
                    WorldChanges::topology()
                }
                AddRandomPeersToEachNode(new_peers_min, new_peers_max) => {
                    let nodes = all_nodes(world);
                    for node in nodes.into_iter() {
                        self.add_random_nodes_as_peers(world, node, new_peers_min, new_peers_max);
                    }
                    WorldChanges::topology()
                }
                MakeDelaunayNetwork => {
                    make_delaunay_network(world);
                    WorldChanges::topology()
                }
            },
        }
    }
    fn log(&mut self, sim_time: SimSeconds, message: String) {
        self.message_log.push_front((sim_time, message));
        self.message_log.truncate(12);
    }
}
