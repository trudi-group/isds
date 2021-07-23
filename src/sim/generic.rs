use super::*;

pub struct TimeSpan {
    pub start: SimSeconds,
    pub end: SimSeconds,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PeerSet(pub BTreeSet<Entity>);

pub fn pick_random_node(world: &mut World, rng: &mut impl Rng) -> Option<Entity> {
    all_nodes(world).choose(rng).map(|&id| id)
}

pub fn pick_random_other_node(
    world: &mut World,
    rng: &mut impl Rng,
    node: Entity,
) -> Option<Entity> {
    all_other_nodes(world, node).choose(rng).map(|&id| id)
}

pub fn pick_random_peer(world: &mut World, rng: &mut impl Rng, node: Entity) -> Option<Entity> {
    peers(world, node).0.iter().choose(rng).map(|&id| id)
}

pub fn all_nodes(world: &mut World) -> Vec<Entity> {
    world
        .query_mut::<&UnderlayNodeName>()
        .into_iter()
        .map(|(id, _)| id)
        .collect()
}

fn all_other_nodes(world: &mut World, node: Entity) -> Vec<Entity> {
    world
        .query_mut::<&UnderlayNodeName>()
        .into_iter()
        .map(|(id, _)| id)
        .filter(|id| *id != node)
        .collect()
}

impl Simulator {
    pub fn add_random_nodes_as_peers(
        &mut self,
        world: &mut World,
        node: Entity,
        new_peers_min: usize,
        new_peers_max: usize,
    ) {
        let mut candidates = all_other_nodes(world, node);
        let mut peers = peers(world, node);
        candidates.retain(|id| !peers.0.contains(id));

        let new_peers_min = cmp::min(new_peers_min, candidates.len());
        let new_peers_max = cmp::min(new_peers_max, candidates.len());
        let number_of_new_peers = self.rng.gen_range(new_peers_min..new_peers_max);

        let new_peers = candidates.choose_multiple(&mut self.rng, number_of_new_peers);
        for node in new_peers {
            peers.0.insert(*node);
        }
    }
    pub fn send_message_to_random_node(
        &mut self,
        world: &mut World,
        start_time: SimSeconds,
        source: Entity,
    ) -> Result<Entity, &str> {
        if let Some(dest) = pick_random_other_node(world, &mut self.rng, source) {
            Ok(self.send_message(world, start_time, source, dest))
        } else {
            Err("Couldn't find a suitable message destination. Not enough nodes around?")
        }
    }
    pub fn send_message_to_random_peer(
        &mut self,
        world: &mut World,
        start_time: SimSeconds,
        source: Entity,
    ) -> Result<Entity, &str> {
        if let Some(dest) = pick_random_peer(world, &mut self.rng, source) {
            Ok(self.send_message(world, start_time, source, dest))
        } else {
            Err("Couldn't find a suitable message destination. Not enough peers?")
        }
    }
}

pub fn add_peer(world: &mut World, node: Entity, peer: Entity) {
    peers(world, node).0.insert(peer);
}

pub fn peers<'a>(world: &'a mut World, node: Entity) -> hecs::RefMut<'a, PeerSet> {
    if world.get_mut::<PeerSet>(node).is_err() {
        let peers = PeerSet(BTreeSet::new());
        world.insert_one(node, peers).unwrap();
    }
    world.get_mut::<PeerSet>(node).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn add_peer_adds_peer() {
        let mut world = World::default();
        let mut simulator = Simulator::new();
        let node1 = simulator.spawn_random_node(&mut world);
        let node2 = simulator.spawn_random_node(&mut world);
        add_peer(&mut world, node1, node2);

        let expected = PeerSet(vec![node2].into_iter().collect());
        let actual = (*world.get::<PeerSet>(node1).unwrap()).clone();

        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn add_random_other_nodes_as_peers_adds_peers() {
        let mut world = World::default();
        let mut simulator = Simulator::new();
        let node1 = simulator.spawn_random_node(&mut world);
        simulator.spawn_random_node(&mut world);
        simulator.spawn_random_node(&mut world);
        simulator.spawn_random_node(&mut world);
        simulator.spawn_random_node(&mut world);

        simulator.add_random_nodes_as_peers(&mut world, node1, 2, 3);

        let peers = peers(&mut world, node1);
        let actual = peers.0.len();
        let expected_min = 2;
        let expected_max = 3;

        assert!(expected_min <= actual);
        assert!(actual <= expected_max);
        assert!(!peers.0.contains(&node1));
    }
}
