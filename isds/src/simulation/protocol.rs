use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PokeNode;
impl EntityAction for PokeNode {
    fn execute_for(&self, sim: &mut Simulation, entity: Entity) -> Result<(), Box<dyn Error>> {
        sim.schedule_now(Event::Node(entity, NodeEvent::Poke));
        // TODO do we want to check if it's a node?
        Ok(())
    }
}

/// A convenience command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PokeSpecificNode(pub Entity);
impl Command for PokeSpecificNode {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        ForSpecific(self.0, PokeNode).execute(sim)
    }
}

// A type alias...
pub trait Payload: 'static + Send + Sync + Clone {}
impl<T> Payload for T where T: 'static + Send + Sync + Clone {}

pub trait Protocol {
    type MessagePayload: Payload;

    /// What to do once we got a message (of type `MessagePayload`).
    fn handle_message(
        &self,
        node: NodeInterface,
        underlay_message: UnderlayMessage,
        message_payload: Self::MessagePayload,
    ) -> Result<(), Box<dyn Error>>;

    /// A default action to take on user interaction with the node (such as a click).
    fn handle_poke(&self, mut node: NodeInterface) -> Result<(), Box<dyn Error>> {
        node.log("I just got poked!");
        Ok(())
    }

    /// What to do once the peer set changes. Optional because not every protocol needs peers or
    /// wants to use the default peer set abstraction.
    fn handle_peer_set_update(
        &self,
        _node: NodeInterface,
        _update: PeerSetUpdate,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl Simulation {
    pub fn node_interface(&mut self, node: Entity) -> NodeInterface {
        NodeInterface::new(self, node)
    }
}

pub struct InvokeProtocolForAllNodes<P: Protocol>(pub P);
impl<P: Protocol> InvokeProtocolForAllNodes<P> {
    fn handle_node_event(
        &mut self,
        sim: &mut Simulation,
        node: Entity,
        event: NodeEvent,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            NodeEvent::MessageSent(_) => {
                // I probably either know about this already or it's not my business
            }
            NodeEvent::MessageArrived(message) => {
                if let Ok((underlay_message, payload)) = sim
                    .world
                    .query_one_mut::<(&UnderlayMessage, &P::MessagePayload)>(message)
                {
                    let (underlay_message, payload) = (*underlay_message, payload.clone());
                    // sim.log(format!(
                    //     "{}: Got message from {}",
                    //     sim.name(node),
                    //     sim.name(underlay_message.source),
                    // ));
                    self.0
                        .handle_message(sim.node_interface(node), underlay_message, payload)?;
                }
                // not my message payload, not my business
            }
            NodeEvent::TimerFired(_) => {
                todo!();
            }
            NodeEvent::PeerSetChanged(update) => {
                self.0
                    .handle_peer_set_update(sim.node_interface(node), update)?;
            }
            NodeEvent::Poke => {
                // sim.log(format!("{}: Got poked!", sim.name(node)));
                self.0.handle_poke(sim.node_interface(node))?;
            }
        }
        Ok(())
    }
}
impl<P: Protocol> EventHandler for InvokeProtocolForAllNodes<P> {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>> {
        if let Event::Node(node, event) = event {
            self.handle_node_event(sim, node, event)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random_walks::RandomWalks;
    use crate::simple_flooding::{SimpleFlooding, SimpleFloodingState};
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn invoking_two_protocols_for_all_nodes_is_possible() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(SimpleFlooding::<u32>::new()));
        sim.add_event_handler(InvokeProtocolForAllNodes(RandomWalks::new(23)));

        sim.do_now(SpawnRandomNodes(8));
        sim.do_now(MakeDelaunayNetwork);
        sim.catch_up(10.);

        let flooded_value: u32 = 42;
        let start_node = sim.pick_random_node().unwrap();
        SimpleFlooding::flood(&mut sim.node_interface(start_node), flooded_value);
        sim.do_now(PokeSpecificNode(start_node)); // will start a random walk

        sim.catch_up(1000.);

        let test_node = sim.pick_random_other_node(start_node).unwrap();
        assert!(sim
            .world
            .query_one_mut::<&SimpleFloodingState<u32>>(test_node)
            .unwrap()
            .own_haves
            .contains(&flooded_value));
    }
}
