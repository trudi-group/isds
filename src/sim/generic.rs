use super::*;
use std::collections::BTreeSet;

// FIXME rather: "generic"?

pub struct TimeSpan {
    pub start: SimSeconds,
    pub end: SimSeconds,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PeerSet(pub BTreeSet<Entity>);

impl Simulator {
    pub fn form_directed_link_between_two_random_nodes(
        &mut self,
        world: &mut World,
    ) -> Result<(Entity, Entity), &str> {
        let node_ents: Vec<Entity> = world
            .query_mut::<&UnderlayNodeName>()
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        if node_ents.len() < 2 {
            Err("Not enough nodes around.")
        } else {
            let selected_node_ids: Vec<Entity> = node_ents
                .choose_multiple(&mut self.rng, 2)
                .copied()
                .collect();
            let source = selected_node_ids[0];
            let dest = selected_node_ids[1];
            self.add_peer(world, source, dest);
            Ok((source, dest))
        }
    }
    pub fn add_peer(&mut self, world: &mut World, node: Entity, peer: Entity) {
        if let Ok(mut peers) = world.get_mut::<PeerSet>(node) {
            peers.0.insert(peer);
            return;
        }
        let mut peers = PeerSet(BTreeSet::new());
        peers.0.insert(peer);
        world.insert_one(node, peers).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn add_peer_forms_a_directed_link() {
        let mut world = World::default();
        let mut simulator = Simulator::new();
        let node1 = simulator.spawn_random_node(&mut world);
        let node2 = simulator.spawn_random_node(&mut world);
        simulator.add_peer(&mut world, node1, node2);

        let expected = PeerSet(vec![node2].into_iter().collect());
        let actual = (*world.get::<PeerSet>(node1).unwrap()).clone();

        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn form_directed_link_between_two_random_nodes_forms_exactly_one_directed_link() {
        let mut world = World::default();
        let mut simulator = Simulator::new();
        simulator.spawn_random_node(&mut world);
        simulator.spawn_random_node(&mut world);

        assert!(world
            .query_mut::<&PeerSet>()
            .into_iter()
            .all(|(_, peer_set)| peer_set.0.is_empty()));
        simulator
            .form_directed_link_between_two_random_nodes(&mut world)
            .unwrap();
        assert_eq!(
            1,
            world
                .query_mut::<&PeerSet>()
                .into_iter()
                .filter(|(_, peer_set)| !peer_set.0.is_empty())
                .count()
        );
    }
}
