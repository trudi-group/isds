use super::*;

use std::cmp;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PeerSetUpdate {
    PeerAdded(Entity),
    PeerRemoved(Entity),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddPeer(pub Entity, pub Entity);
impl Command for AddPeer {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        sim.add_peer(self.0, self.1);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemovePeer(pub Entity, pub Entity);
impl Command for RemovePeer {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        sim.remove_peer(self.0, self.1);
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MakeDelaunayNetwork;
impl Command for MakeDelaunayNetwork {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        sim.make_delaunay_network();
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PeerSet {
    peers: BTreeSet<Entity>,
    last_update: SimSeconds, // for helping the UI know when to redraw
}
impl PeerSet {
    /// Only useful for tests really.
    pub fn default_from(peers: impl IntoIterator<Item = Entity>) -> Self {
        Self {
            peers: peers.into_iter().collect(),
            last_update: Default::default(),
        }
    }
    pub fn iter(&self) -> std::collections::btree_set::Iter<Entity> {
        self.peers.iter()
    }
    pub fn insert(&mut self, peer: Entity, now: SimSeconds) -> bool {
        if self.peers.insert(peer) {
            self.last_update = now;
            true
        } else {
            false
        }
    }
    pub fn remove(&mut self, peer: &Entity, now: SimSeconds) -> bool {
        if self.peers.remove(peer) {
            self.last_update = now;
            true
        } else {
            false
        }
    }
    pub fn contains(&self, peer: &Entity) -> bool {
        self.peers.contains(peer)
    }
    pub fn len(&self) -> usize {
        self.peers.len()
    }
    pub fn last_update(&self) -> SimSeconds {
        self.last_update
    }
}
impl IntoIterator for PeerSet {
    type Item = Entity;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.peers.into_iter()
    }
}

impl Simulation {
    pub fn peers_mut(&mut self, node: Entity) -> hecs::RefMut<PeerSet> {
        if self.world.get_mut::<PeerSet>(node).is_err() {
            let peers = PeerSet::default();
            self.world.insert_one(node, peers).unwrap();
        }
        self.world.get_mut::<PeerSet>(node).unwrap()
    }
    pub fn add_peer(&mut self, node: Entity, peer: Entity) {
        let now = self.time.now();
        self.peers_mut(node).insert(peer, now);
        self.schedule_now(Event::Node(
            node,
            NodeEvent::PeerSetChanged(PeerSetUpdate::PeerAdded(peer)),
        ));
    }
    pub fn remove_peer(&mut self, node: Entity, peer: Entity) {
        let now = self.time.now();
        self.peers_mut(node).remove(&peer, now);
        self.schedule_now(Event::Node(
            node,
            NodeEvent::PeerSetChanged(PeerSetUpdate::PeerRemoved(peer)),
        ));
    }
    pub fn add_random_nodes_as_peers(
        &mut self,
        node: Entity,
        new_peers_min: usize,
        new_peers_max: usize,
    ) {
        let mut candidates = self.all_other_nodes(node);
        let peers = self.peers_mut(node).clone();
        candidates.retain(|id| !peers.contains(id));

        let new_peers_min = cmp::min(new_peers_min, candidates.len());
        let new_peers_max = cmp::min(new_peers_max, candidates.len());
        let number_of_new_peers = self.rng.gen_range(new_peers_min..new_peers_max);

        let new_peers = candidates.choose_multiple(&mut self.rng, number_of_new_peers);
        for &peer in new_peers {
            self.add_peer(node, peer);
        }
    }

    fn make_delaunay_network(&mut self) {
        use delaunator::{triangulate, Point};
        let (nodes, points): (Vec<Entity>, Vec<Point>) = self
            .world
            .query_mut::<(&UnderlayNodeName, &UnderlayPosition)>()
            .into_iter()
            .map(|(id, (_, pos))| {
                (
                    id,
                    Point {
                        x: pos.x as f64,
                        y: pos.y as f64,
                    },
                )
            })
            .unzip();
        for &node in nodes.iter() {
            *self.peers_mut(node) = PeerSet::default();
        }
        let triangles = triangulate(&points).triangles;
        assert!(triangles.len() % 3 == 0);
        for i in (0..triangles.len()).step_by(3) {
            let node1 = nodes[triangles[i]];
            let node2 = nodes[triangles[i + 1]];
            let node3 = nodes[triangles[i + 2]];
            self.add_peer(node1, node2);
            self.add_peer(node1, node3);
            self.add_peer(node2, node1);
            self.add_peer(node2, node3);
            self.add_peer(node3, node1);
            self.add_peer(node3, node2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn add_peer_adds_peer() {
        let mut sim = Simulation::new();
        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        sim.add_peer(node1, node2);

        let expected = PeerSet {
            peers: vec![node2].into_iter().collect(),
            last_update: Default::default(),
        };
        let actual = (*sim.world.get::<PeerSet>(node1).unwrap()).clone();

        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn add_random_other_nodes_as_peers_adds_peers() {
        let mut sim = Simulation::new();
        let node1 = sim.spawn_random_node();
        sim.spawn_random_node();
        sim.spawn_random_node();
        sim.spawn_random_node();
        sim.spawn_random_node();

        sim.add_random_nodes_as_peers(node1, 2, 3);

        let peers = sim.peers_mut(node1);
        let actual = peers.len();
        let expected_min = 2;
        let expected_max = 3;

        assert!(expected_min <= actual);
        assert!(actual <= expected_max);
        assert!(!peers.contains(&node1));
    }
}
