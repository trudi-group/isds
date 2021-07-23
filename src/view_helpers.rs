#![allow(clippy::cast_possible_truncation)]
use super::*;
use time::*;
use view::*;

#[derive(Debug, Default)]
pub struct ViewCache {
    last_update: SimSeconds,
    edges: EdgeMap,
    // topology: Vec<Node<Msg>>,
    // messages: Vec<Node<Msg>>,
}
impl ViewCache {
    pub fn new() -> Self {
        Self {
            last_update: OrderedFloat(f64::MIN), // TODO: call this SimSeconds::never(),
            edges: BTreeMap::new(),
            // topology: Vec::new(),
            // messages: Vec::new(),
        }
    }
    // pub fn topology<'a>(&'a self) -> &'a [Node<Msg>] {
    //     &self.topology
    // }
    // pub fn messages<'a>(&'a self) -> &'a [Node<Msg>] {
    //     &self.messages
    // }
    pub fn edges<'a>(&'a self) -> &EdgeMap {
        &self.edges
    }
    pub fn update(&mut self, world: &mut World, sim_time: SimSeconds, changes: WorldChanges) {
        if sim_time == self.last_update {
            return;
        }
        self.last_update = sim_time;

        if changes.topology {
            self.update_topology(world);
        }
        self.update_messages(world, sim_time);
    }
    fn update_topology(&mut self, world: &World) {
        self.rebuild_edges(world);
        // self.topology = view_topology(world, &self.edges);
    }
    fn update_messages(&mut self, world: &mut World, sim_time: SimSeconds) {
        update_message_positions(world, sim_time);
        // self.messages = view_messages(world);
    }
    fn rebuild_edges(&mut self, world: &World) {
        let edges = &mut self.edges;
        edges.clear();

        for (node, peer_set) in world.query::<&PeerSet>().iter() {
            for &peer in peer_set.0.iter() {
                let endpoints = EdgeEndpoints::new(node, peer);
                if edges.contains_key(&endpoints) {
                    let (_type, _) = edges.get_mut(&endpoints).unwrap();
                    *_type = EdgeType::Undirected;
                } else {
                    let _type = if endpoints.left == node {
                        EdgeType::LeftRight
                    } else {
                        EdgeType::RightLeft
                    };
                    let line = UnderlayLine::from_nodes(world, node, peer);
                    edges.insert(endpoints, (_type, line));
                }
            }
        }
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

// FIXME rather somewhere else?
pub fn name(world: &World, entity: Entity) -> String {
    if let Ok(entity_ref) = world.entity(entity) {
        if let Some(node_name) = entity_ref.get::<UnderlayNodeName>() {
            format!("{}", node_name.0)
        } else {
            format!("UNNAMEABLE ({})", entity.id())
        }
    } else {
        format!("INEXISTING ({})", entity.id())
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
