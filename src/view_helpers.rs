#![allow(clippy::cast_possible_truncation)]
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct ViewCache {
    pub edges: BTreeMap<EdgeEndpoints, (EdgeType, UnderlayLine)>,
}
impl ViewCache {
    pub fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
        }
    }
}

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

pub fn update_view_data(world: &mut World, view_cache: &mut ViewCache, sim_time: SimSeconds, changes: WorldChanges) {
    if changes.topology {
        update_connection_lines(world, view_cache);
    }
    update_message_positions(world, sim_time);
}

fn update_connection_lines(world: &World, view_cache: &mut ViewCache) {
    let edges = &mut view_cache.edges;
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

// FIXME rather somewhere in lib?
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
    fn update_connection_lines_updates_view_cache() {
        let mut world = World::default();
        let mut view_cache = ViewCache::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet(vec![node1].into_iter().collect()),
            UnderlayPosition::new(13., 13.),
        ));

        update_connection_lines(&world, &mut view_cache);

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

        update_connection_lines(&world, &mut view_cache);

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
