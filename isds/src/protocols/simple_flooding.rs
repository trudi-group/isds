use super::*;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct SimpleFlooding<T: Payload> {
    payload_type: PhantomData<T>,
}
impl<T: Payload> SimpleFlooding<T> {
    pub fn new() -> Self {
        Self {
            payload_type: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartSimpleFlooding<T: Payload + Default + Hash + Eq>(pub Entity, pub T);
impl<T: Payload + Default + Hash + Eq> Command for StartSimpleFlooding<T> {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        SimpleFlooding::flood(&mut sim.node_interface(self.0), self.1.clone());
        Ok(())
    }
}

impl<T: Payload + Default + Hash + Eq> Protocol for SimpleFlooding<T> {
    type MessagePayload = SimpleFloodingMessage<T>;

    fn handle_message(
        &self,
        mut node: NodeInterface,
        underlay_message: UnderlayMessage,
        message_payload: Self::MessagePayload,
    ) -> Result<(), Box<dyn Error>> {
        let message = message_payload.0;
        register_sender(&mut node, &message, underlay_message.source);
        if is_new(&mut node, &message) {
            Self::flood(&mut node, message);
        }
        Ok(())
    }
    fn handle_poke(&self, mut node: NodeInterface) -> Result<(), Box<dyn Error>> {
        node.log("Got poked. So what? Will init my state at least.");
        node.get::<SimpleFloodingState<T>>();
        Ok(())
    }
    fn handle_peer_set_update(
        &self,
        mut node: NodeInterface,
        update: PeerSetUpdate,
    ) -> Result<(), Box<dyn Error>> {
        match update {
            PeerSetUpdate::PeerAdded(peer) => {
                let own_haves = node.get::<SimpleFloodingState<T>>().own_haves.clone();
                Self::flood_peer_with(&mut node, peer, own_haves)
            }
            PeerSetUpdate::PeerRemoved(peer) => Self::forget_peer(&mut node, peer),
        };
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct SimpleFloodingMessage<T>(pub T);

// TODO: also clear messages from seen set at some point? or isn't that "simple" anymore?
#[derive(Debug, Default, Clone)]
pub struct SimpleFloodingState<T> {
    pub own_haves: HashSet<T>,
    peer_haves: HashMap<Entity, HashSet<T>>,
}

impl<T: Payload + Default + Hash + Eq> SimpleFlooding<T> {
    pub fn flood(node: &mut NodeInterface, message: T) {
        let peers = node.get::<PeerSet>().0.clone(); // TODO: again, the clone here is not ideal
        let flooding_state = node.get::<SimpleFloodingState<T>>();

        let mut next_hops = vec![];

        flooding_state.own_haves.insert(message.clone());
        for peer in peers.into_iter() {
            match flooding_state.peer_haves.entry(peer) {
                Entry::Occupied(mut e) => {
                    if e.get_mut().insert(message.clone()) {
                        next_hops.push(peer);
                    }
                }
                Entry::Vacant(e) => {
                    let mut new_set = HashSet::new();
                    new_set.insert(message.clone());
                    e.insert(new_set);
                    next_hops.push(peer);
                }
            }
        }
        for peer in next_hops.into_iter() {
            node.send_message(peer, SimpleFloodingMessage(message.clone()));
        }
    }
    pub fn forget_peer(node: &mut NodeInterface, peer: Entity) {
        let flooding_state = node.get::<SimpleFloodingState<T>>();
        flooding_state.peer_haves.remove(&peer);
    }
    pub fn flood_peer_with(
        node: &mut NodeInterface,
        peer: Entity,
        items: impl IntoIterator<Item = T> + Clone,
    ) {
        node.get::<SimpleFloodingState<T>>()
            .peer_haves
            .entry(peer)
            .or_default()
            .extend(items.clone().into_iter());
        node.send_messages(peer, items.into_iter().map(SimpleFloodingMessage));
    }
}

fn is_new<T: Payload + Default + Hash + Eq>(node: &mut NodeInterface, message: &T) -> bool {
    let flooding_state = node.get::<SimpleFloodingState<T>>();
    !flooding_state.own_haves.contains(message)
}

fn register_sender<T: Payload + Default + Hash + Eq>(
    node: &mut NodeInterface,
    message: &T,
    sender: Entity,
) {
    let flooding_state = node.get::<SimpleFloodingState<T>>();
    match flooding_state.peer_haves.entry(sender) {
        Entry::Occupied(mut e) => {
            e.get_mut().insert(message.clone());
        }
        Entry::Vacant(e) => {
            let mut new_set = HashSet::new();
            new_set.insert(message.clone());
            e.insert(new_set);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn simple_flooding_floods() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(SimpleFlooding::<u32>::new()));

        sim.do_now(SpawnRandomNodes(8));
        sim.do_now(MakeDelaunayNetwork);
        sim.catch_up(1.);

        let flooded_value: u32 = 42;
        let start_node = sim.pick_random_node().unwrap();
        SimpleFlooding::flood(&mut sim.node_interface(start_node), flooded_value);

        sim.catch_up(1000.);

        let received_values: Vec<HashSet<u32>> = sim
            .world
            .query_mut::<&SimpleFloodingState<u32>>()
            .into_iter()
            .map(|(_, s)| s.own_haves.clone())
            .collect();
        assert!(!received_values.is_empty());

        let as_expected_nodes = received_values
            .into_iter()
            .filter(|v| v.contains(&flooded_value) && v.len() == 1);
        assert_eq!(8, as_expected_nodes.count());
    }

    #[wasm_bindgen_test]
    fn simple_flooding_recovers_from_splits() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(SimpleFlooding::<u32>::new()));

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();

        let flooded_value: u32 = 42;

        SimpleFlooding::flood(&mut sim.node_interface(node1), flooded_value);

        sim.catch_up(1000.);

        add_peer(&mut sim, node1, node2);

        sim.catch_up(1000.);

        assert!(sim
            .world
            .query_one_mut::<&SimpleFloodingState<u32>>(node2)
            .unwrap()
            .own_haves
            .contains(&flooded_value));
    }
}
