use super::*;

use hecs::QueryItem;

pub mod blockchain_types;

pub struct NodeInterface<'a> {
    sim: &'a mut Simulation,
    node: Entity,
}
impl<'a> NodeInterface<'a> {
    pub fn new(sim: &'a mut Simulation, node: Entity) -> Self {
        Self { sim, node }
    }
    pub fn get<T: Payload + Default>(&mut self) -> QueryItem<&mut T> {
        if self.sim.world.query_one_mut::<&T>(self.node).is_err() {
            self.sim.world.insert_one(self.node, T::default()).unwrap();
        }
        self.sim.world.query_one_mut::<&mut T>(self.node).unwrap()
    }
    pub fn log(&mut self, message: &str) {
        self.sim
            .log(format!("{}: {}", self.sim.name(self.node), message));
    }
    pub fn send_message<P: Payload>(&mut self, dest: Entity, payload: P) -> Entity {
        let source = self.node;
        self.sim.send_message(source, dest, payload)
    }
    pub fn send_messages<P: Payload>(
        &mut self,
        dest: Entity,
        payloads: impl IntoIterator<Item = P>,
    ) -> Vec<Entity> {
        let source = self.node;
        self.sim.send_messages(source, dest, payloads)
    }
    pub fn rng(&mut self) -> &mut impl Rng {
        &mut self.sim.rng
    }
}
