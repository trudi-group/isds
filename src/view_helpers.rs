#![allow(clippy::cast_possible_truncation)]

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

#[derive(Debug, Clone)]
pub struct ViewCache {
    colors: PseudorandomColors,
    edges: EdgeMap,
}
impl Default for ViewCache {
    fn default() -> Self {
        let seed_colors = &[
            // WI colors
            "#A69D82", // greige
            "#7D505A", // mauve
            "#235A82", // blue
            "#46695A", // darkgreen
            "#829664", // lightgreen
            "#C88C28", // yellow
            "#BE552D", // orange
        ];
        Self::new(seed_colors, 64)
    }
}
impl ViewCache {
    pub fn new(seed_palette: &[&str], target_palette_n: usize) -> Self {
        let colors = PseudorandomColors::new(seed_palette, target_palette_n);
        Self {
            colors,
            edges: BTreeMap::new(),
        }
    }
    pub fn edges(&self) -> &EdgeMap {
        &self.edges
    }
    pub fn edge_type(&self, endpoint1: Entity, endpoint2: Entity) -> Option<EdgeType> {
        self.edges
            .get(&EdgeEndpoints::new(endpoint1, endpoint2))
            .map(|e| e.0)
    }
    pub fn color(&self, number: u32) -> &str {
        self.colors.get(number)
    }
    pub fn colors(&self) -> &[String] {
        self.colors.all()
    }
    fn rebuild_edges(&mut self, world: &World) {
        let edges = &mut self.edges;

        for (edge_type, _) in edges.values_mut() {
            *edge_type = EdgeType::Phantom;
        }

        log!("Rebuilding edges...");

        for (node, peer_set) in world.query::<&PeerSet>().iter() {
            for &peer in peer_set.0.iter() {
                let endpoints = EdgeEndpoints::new(node, peer);
                match edges.entry(endpoints) {
                    Entry::Occupied(mut e) => {
                        let e = e.get_mut();
                        if e.0 == EdgeType::Phantom {
                            e.0 = if endpoints.left == node {
                                EdgeType::LeftRight
                            } else {
                                EdgeType::RightLeft
                            };
                        } else {
                            e.0 = EdgeType::Undirected;
                        }
                    }
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
impl sim::EventWatcher for ViewCache {
    fn handle_event(&mut self, sim: &Simulation, event: Event) -> Result<(), Box<dyn Error>> {
        if let Event::Command(_) = event {
            // TODO: We probably don't want to do this *that* often.
            self.rebuild_edges(&sim.world);
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
impl EdgeEndpoints {
    pub fn new(node1: Entity, node2: Entity) -> Self {
        let (left, right) = if node1 <= node2 {
            (node1, node2)
        } else {
            (node2, node1)
        };
        Self { left, right }
    }
    pub fn left(&self) -> Entity {
        self.left
    }
    pub fn right(&self) -> Entity {
        self.right
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EdgeType {
    Undirected,
    LeftRight,
    RightLeft,
    Phantom,
}
impl EdgeType {
    pub fn is_phantom(&self) -> bool {
        *self == Self::Phantom
    }
}

#[derive(Debug, Clone)]
pub struct PseudorandomColors {
    full_palette: Vec<String>,
}

impl PseudorandomColors {
    pub fn new(seed_palette: &[&str], target_palette_n: usize) -> Self {
        use palette::{FromColor, Gradient, Lab, Pixel, Srgb};
        use std::str::FromStr;
        assert!(seed_palette.len() <= target_palette_n);

        let seed_colors = seed_palette.iter().map(|c| Srgb::from_str(c).unwrap());
        let gradient = Gradient::new(
            seed_colors.map(|c| Lab::from_color(c.into_format::<f32>().into_linear())),
        );

        let full_palette = gradient
            .take(target_palette_n)
            .map(|c| {
                format!(
                    "#{}",
                    hex::encode(Srgb::from_color(c).into_format().into_raw::<[u8; 3]>())
                )
            })
            .collect();

        Self { full_palette }
    }
    pub fn get(&self, number: u32) -> &str {
        let index = pseudorandomize(number) as usize % self.full_palette.len();
        &self.full_palette[index]
    }
    pub fn all(&self) -> &[String] {
        &self.full_palette
    }
}

pub fn pseudorandomize(number: u32) -> u32 {
    // inspired by legion's `U64Hasher`
    let big_prime = 2u32.pow(31) - 1; // eighth Mersenne prime, largest prime in `u32`
    big_prime.wrapping_mul(number)
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
    fn rebuild_edges_sets_direction() {
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

    #[wasm_bindgen_test]
    fn rebuild_edges_stores_removed_edges_as_phantom_edges() {
        let mut world = World::default();
        let mut view_cache = ViewCache::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet(vec![node1].into_iter().collect()),
            UnderlayPosition::new(13., 13.),
        ));

        view_cache.rebuild_edges(&world);

        world
            .query_one_mut::<&mut PeerSet>(node2)
            .unwrap()
            .0
            .remove(&node1);

        view_cache.rebuild_edges(&world);

        assert_eq!(
            EdgeType::Phantom,
            view_cache
                .edges
                .get(&EdgeEndpoints::new(node1, node2))
                .unwrap()
                .0,
        );
    }

    #[wasm_bindgen_test]
    fn parse_one_html_color() {
        let colors = PseudorandomColors::new(&["#008000"], 1);
        let expected = vec!["#008000"];
        let actual = colors.full_palette;
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn blend_between_multiple_html_colors() {
        let colors = PseudorandomColors::new(&["#008000", "#0000FF", "#ffff00"], 5);
        assert_eq!(colors.full_palette[0], "#008000");
        assert_ne!(colors.full_palette[1], "#008000");
        assert_ne!(colors.full_palette[1], "#0000ff");
        assert_eq!(colors.full_palette[2], "#0000ff");
        assert_ne!(colors.full_palette[3], "#0000ff");
        assert_ne!(colors.full_palette[3], "#ffff00");
        assert_eq!(colors.full_palette[4], "#ffff00");
    }

    #[wasm_bindgen_test]
    fn pseudorandom_is_random_but_deterministic() {
        let colors = PseudorandomColors::new(&["#008000", "#0000FF", "#ffff00"], 1024);
        assert_eq!(colors.get(42), colors.get(42));
        assert_ne!(colors.get(23), colors.get(42));
    }
}
