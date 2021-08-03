use hecs::QueryItem;

use super::*;

pub struct NodeInterface<'a> {
    sim: &'a mut Simulation,
    node: Entity,
}
impl<'a> NodeInterface<'a> {
    pub fn new(sim: &'a mut Simulation, node: Entity) -> Self {
        Self { sim, node }
    }
    pub fn get<T: 'static + Sync + Send + Default>(&mut self) -> QueryItem<&T> {
        if self.sim.world.query_one_mut::<&T>(self.node).is_err() {
            self.sim.world.insert_one(self.node, T::default()).unwrap();
        }
        self.sim.world.query_one_mut::<&T>(self.node).unwrap()
    }
    pub fn log(&mut self, message: &str) {
        self.sim
            .log(format!("{}: {}", self.sim.name(self.node), message));
    }
    pub fn send_message<P: hecs::Component>(&mut self, dest: Entity, payload: P) -> Entity {
        let source = self.node;
        self.sim.send_message(source, dest, payload)
    }
    pub fn rng(&mut self) -> &mut impl Rng {
        &mut self.sim.rng
    }
}

pub trait Protocol {
    type MessagePayload: 'static + Send + Sync + Clone;

    fn handle_message(
        &self,
        node: NodeInterface,
        underlay_message: UnderlayMessage,
        message_payload: Self::MessagePayload,
    ) -> Result<(), Box<dyn Error>>;

    fn handle_poke(&self, node: NodeInterface) -> Result<(), Box<dyn Error>>;

    fn handle_node_event(
        &mut self,
        sim: &mut Simulation,
        node: Entity,
        event: NodeEvent,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            NodeEvent::MessageArrived(message) => {
                let (underlay_message, payload) = sim
                    .world
                    .query_one_mut::<(&UnderlayMessage, &Self::MessagePayload)>(message)
                    .unwrap();
                let (underlay_message, payload) = (*underlay_message, payload.clone());
                sim.log(format!(
                    "{}: Got message from {}",
                    sim.name(node),
                    sim.name(underlay_message.source),
                ));
                self.handle_message(NodeInterface::new(sim, node), underlay_message, payload)?;
            }
            NodeEvent::Poke => {
                sim.log(format!("{}: Got poked!", sim.name(node)));
                self.handle_poke(NodeInterface::new(sim, node))?;
            }
            NodeEvent::TimerFired(_) => {
                todo!();
            }
        }
        Ok(())
    }
}

impl<P: Protocol> EventHandlerMut for P {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>> {
        if let Event::Node(node, event) = event {
            self.handle_node_event(sim, node, event)
        } else {
            Ok(())
        }
    }
}
