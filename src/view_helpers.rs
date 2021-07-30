#![allow(clippy::cast_possible_truncation)]

use protocols::generic::PeerSet;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use super::*;

#[derive(Debug, Default)]
pub struct FPSCounter {
    time_elapsed: f64,
    fps_sample: f64,
}
impl FPSCounter {
    pub fn register_render_interval(&mut self, interval: f64) {
        self.time_elapsed += interval;
        if self.time_elapsed > 0.5 {
            self.fps_sample = 1. / interval;
            self.time_elapsed = 0.;
        }
    }
    pub fn get(&self) -> f64 {
        self.fps_sample
    }
}

#[derive(Debug, Default)]
pub struct ViewCache {
    edges: EdgeMap,
}
impl ViewCache {
    pub fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
        }
    }
    pub fn edges(&self) -> &EdgeMap {
        &self.edges
    }
    pub fn update_messages(&mut self, world: &mut World, sim_time: SimSeconds) {
        update_message_positions(world, sim_time);
        // self.messages = view_messages(world);
    }
    fn rebuild_edges(&mut self, world: &World) {
        let edges = &mut self.edges;
        edges.clear();

        for (node, peer_set) in world.query::<&PeerSet>().iter() {
            for &peer in peer_set.0.iter() {
                let endpoints = EdgeEndpoints::new(node, peer);
                match edges.entry(endpoints) {
                    Entry::Occupied(mut e) => e.get_mut().0 = EdgeType::Undirected,
                    Entry::Vacant(e) => {
                        let _type = if endpoints.left == node {
                            EdgeType::LeftRight
                        } else {
                            EdgeType::RightLeft
                        };
                        let line = UnderlayLine::from_nodes(world, node, peer);
                        e.insert((_type, line));
                    }
                }
            }
        }
    }
}
impl sim::EventHandler for ViewCache {
    fn handle_event(&mut self, sim: &Simulation, event: SimEvent) -> Result<(), Box<dyn Error>> {
        if let SimEvent::ExternalCommand(command) = event {
            if matches!(
                command,
                SimCommand::MakeDelaunayNetwork | SimCommand::AddRandomPeersToEachNode(_, _)
            ) {
                self.rebuild_edges(&sim.world);
            }
        }
        Ok(())
    }
}

pub type EdgeMap = BTreeMap<EdgeEndpoints, (EdgeType, UnderlayLine)>;

#[derive(Debug, Copy, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct EdgeEndpoints {
    left: Entity,
    right: Entity,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EdgeType {
    Undirected,
    LeftRight,
    RightLeft,
}
impl EdgeEndpoints {
    pub fn new(node1: Entity, node2: Entity) -> Self {
        let (left, right) = if node1 <= node2 {
            (node1, node2)
        } else {
            (node2, node1)
        };
        Self { left, right }
    }
}

fn update_message_positions(world: &mut World, sim_time: SimSeconds) {
    for (_, (path, time_span, position)) in
        world.query_mut::<(&UnderlayLine, &TimeSpan, &mut UnderlayPosition)>()
    {
        let progress =
            ((sim_time - time_span.start) / (time_span.end - time_span.start)).into_inner() as f32;
        // clippy said that `mul_add` could be faster...
        position.x = (path.end.x - path.start.x).mul_add(progress, path.start.x);
        position.y = (path.end.y - path.start.y).mul_add(progress, path.start.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn rebuild_edges_builds_edges() {
        let mut world = World::default();
        let mut view_cache = ViewCache::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet(vec![node1].into_iter().collect()),
            UnderlayPosition::new(13., 13.),
        ));

        view_cache.rebuild_edges(&world);

        assert!(view_cache
            .edges
            .contains_key(&EdgeEndpoints::new(node1, node2)));
    }

    #[wasm_bindgen_test]
    fn update_connection_lines_set_direction() {
        let mut world = World::default();
        let mut view_cache = ViewCache::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet(vec![node1].into_iter().collect()),
            UnderlayPosition::new(13., 13.),
        ));

        view_cache.rebuild_edges(&world);

        assert_ne!(
            EdgeType::Undirected,
            view_cache
                .edges
                .get(&EdgeEndpoints::new(node1, node2))
                .unwrap()
                .0
        );
    }
}
