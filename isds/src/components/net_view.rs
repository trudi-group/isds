use super::*;

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;


pub struct NetView {
    sim: Rc<RefCell<Simulation>>,
    colors: PseudorandomColors,
    edges: EdgeMap,
    _context_handle: yew::context::ContextHandle<ContextData>
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Rendered(RealSeconds),
    NodeClick(Entity),
    LinkClick(Entity, Entity),
}

impl Component for NetView {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context_data, _context_handle) = get_context_data!(ctx, Self);

        let sim = context_data.sim;

        // TODO as props?
        let seed_palette = &[
            // WI colors
            "#A69D82", // greige
            "#7D505A", // mauve
            "#235A82", // blue
            "#46695A", // darkgreen
            "#829664", // lightgreen
            "#C88C28", // yellow
            "#BE552D", // orange
        ];
        let target_palette_n = 64;

        let colors = PseudorandomColors::new(seed_palette, target_palette_n);
        let edges = EdgeMap::default();

        Self { sim, colors, edges, _context_handle }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let buffer_space = 50.;
        html!{
            <svg
               viewBox={ format!("{} {} {} {}",
                   -buffer_space,
                   -buffer_space,
                   self.sim.borrow().underlay_width() + 2. * buffer_space,
                   self.sim.borrow().underlay_height() + 2. * buffer_space
                ) }
                style=r#"border-style: "solid"; max-width: 1024px"#
            >
                { self.view_palette() }
                { self.view_edges(ctx) }
                { self.view_nodes(ctx) }
                { self.view_messages() }
            </svg>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(time) => {
                if time < 10. { // FIXME
                    self.rebuild_edges();
                }
                true
            }
            Msg::NodeClick(node) => {
                log!(format!("Click on {}", self.sim.borrow().name(node)));
                self.sim.borrow_mut().do_now(PokeNode(node));
                // self
                //     .sim
                //     .do_now(protocols::simple_flooding::StartSimpleFlooding(
                //         node,
                //         rand::random::<u32>(),
                //     ));
                false
            }
            Msg::LinkClick(node1, node2) => {
                log!(format!(
                    "Click on link between {} and {}.",
                    self.sim.borrow().name(node1),
                    self.sim.borrow().name(node2)
                ));
                if self
                    .edges
                    .edge_type(node1, node2)
                    .unwrap()
                    .is_phantom()
                {
                    self.sim.borrow_mut().do_now(AddPeer(node1, node2));
                    self.sim.borrow_mut().do_now(AddPeer(node2, node1));
                } else {
                    self.sim.borrow_mut().do_now(RemovePeer(node1, node2));
                    self.sim.borrow_mut().do_now(RemovePeer(node2, node1));
                }
                self.rebuild_edges(); // FIXME
                false
            }
        }
    }
}

impl NetView {
    fn rebuild_edges(&mut self) {
        self.edges.rebuild(&self.sim.borrow().world) // FIXME
    }
    fn view_nodes(&self, ctx: &Context<NetView>) -> Html {
        let r = 5.0;
        let link = ctx.link();
        self.sim.borrow().world
            .query::<(&UnderlayPosition, &nakamoto_consensus::NakamotoNodeState)>()
            .into_iter()
            .map(|(node, (pos, node_state))| html! {
                <g>
                    <circle
                        cx={ pos.x.to_string() }
                        cy={ pos.y.to_string() }
                        r={ r.to_string() }
                        onclick={ link.callback(move |_| Msg::NodeClick(node)) }
                    />
                    { self.view_blocks(node_state, pos.x + 8., pos.y - 8.) }
                </g>
            })
            .collect()
    }
    fn view_edges(&self, ctx: &Context<NetView>) -> Html {
        let link = ctx.link();
        self
            .edges
            .0
            .iter()
            .map(|(&edge_endpoints, &(edge_type, line))| html! {
                <g
                    onclick={ link.callback(move |_| Msg::LinkClick(
                        edge_endpoints.left(),
                        edge_endpoints.right()
                    )) }
                >
                    <line
                        class="phantom-link"
                        x1={ line.start.x.to_string() }
                        y1={ line.start.y.to_string() }
                        x2={ line.end.x.to_string() }
                        y2={ line.end.y.to_string() }
                        stroke="yellow"
                        stroke-width=8
                    />
                    if edge_type != EdgeType::Phantom {
                        if edge_type == EdgeType::Undirected {
                            <line
                                x1={ line.start.x.to_string() }
                                y1={ line.start.y.to_string() }
                                x2={ line.end.x.to_string() }
                                y2={ line.end.y.to_string() }
                                stroke="gray"
                            />
                        } else {
                            // TODO: https://developer.mozilla.org/en-US/docs/Web/SVG/Element/marker
                            <line
                                x1={ line.start.x.to_string() }
                                y1={ line.start.y.to_string() }
                                x2={ line.end.x.to_string() }
                                y2={ line.end.y.to_string() }
                                stroke="lightgray"
                                stroke-dasharray="8,8"
                            />
                        }
                    }
                </g>
            })
            .collect()
    }
    fn view_messages(&self) -> Html {
        let time_now = self.sim.borrow().time.now();
        self.sim.borrow().world
            .query::<(
                &UnderlayLine,
                &TimeSpan,
                &simple_flooding::SimpleFloodingMessage<nakamoto_consensus::Block>,
            )>()
            .into_iter()
            .map(|(_, (trajectory, time_span, message))| {
                let (x, y) = message_position(trajectory, time_span, time_now);
                let block = message.0;
                html! {
                    <circle
                        cx={ x.to_string() }
                        cy={ y.to_string() }
                        r=2
                        fill={ self.colors.get(nakamoto_consensus::to_number(block.hash())).to_string() }
                    />
                }
            }).collect()
    }
    fn view_blocks(
        &self,
        state: &nakamoto_consensus::NakamotoNodeState,
        x: f32,
        y: f32,
    ) -> Html {
        let max_depth = 5;
        let block_height = 5.;
        let block_width = 5.;
        let block_spacing = 2.;

        let block_map = block_map(state, max_depth);
        let mut result = vec![];

        for i in 0..block_map.len() {
            for j in 0..block_map[i].len() {
                if let Some(block_hash) = block_map[i][j] {
                    if let Some(k) = block_map.iter().take(i).enumerate().find_map(|(k, chain)| {
                        if let Some(other_chain_hash) = chain[j] {
                            if other_chain_hash == block_hash {
                                Some(k)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }) {
                        result.push(
                            html! {
                                <line
                                    x1={ (x + (block_width + block_spacing) * (i as f32) + block_width / 2.).to_string() }
                                    x2={ (x + (block_width + block_spacing) * (k as f32) + block_width).to_string() }
                                    y1={ (y + (block_height + block_spacing) * (j as f32)).to_string() }
                                    y2={ (y + (block_height + block_spacing) * (j as f32) + block_height /2.).to_string() }
                                    stroke={ (self.colors.get(nakamoto_consensus::to_number(block_map[i][j-1].unwrap()))).to_string() }
                                />
                            }
                        );
                        break;
                    } else {
                        result.push(html! {
                            <rect
                                x={ (x + (block_width + block_spacing)* (i as f32)).to_string() }
                                y={ (y + (block_height + block_spacing)* (j as f32)).to_string() }
                                width={ (block_width).to_string() }
                                height={ (block_height).to_string() }
                                fill={ self.colors.get(nakamoto_consensus::to_number(block_hash)).to_string() }
                            />
                        });
                        result.push(html! {
                            <line
                                x1={ (x + (block_width + block_spacing) * (i as f32) + block_width / 2.).to_string() }
                                x2={ (x + (block_width + block_spacing) * (i as f32) + block_width / 2.).to_string() }
                                y1={ (y + (block_height + block_spacing) * (j as f32) + block_height).to_string() }
                                y2={ (y + (block_height + block_spacing) * ((j + 1) as f32)).to_string() }
                                stroke={ (self.colors.get(nakamoto_consensus::to_number(block_hash))).to_string() }
                            />
                        });
                    }
                }
            }
        }
        result.into_iter().collect()
    }
    pub fn view_palette(&self) -> Html {
        self
            .colors
            .all()
            .iter()
            .enumerate()
            .map(|(i, color)| html! {
                <circle
                    cx={ format!("{}", -40 + 5 * (i as i32)) }
                    cy={ format!("{}", -40) }
                    r=5
                    fill={ color.clone() }
                />
                }).collect()
    }
}

fn message_position(
    trajectory: &UnderlayLine,
    time_span: &TimeSpan,
    time_now: SimSeconds,
) -> (f32, f32) {
    let progress = time_span.progress_clamped(time_now) as f32;
    // clippy said that `mul_add` could be faster...
    let x = (trajectory.end.x - trajectory.start.x).mul_add(progress, trajectory.start.x);
    let y = (trajectory.end.y - trajectory.start.y).mul_add(progress, trajectory.start.y);
    (x, y)
}

#[derive(Debug, Default)]
struct EdgeMap(BTreeMap<EdgeEndpoints, (EdgeType, UnderlayLine)>);
impl EdgeMap {
    fn rebuild(&mut self, world: &World) {
        let edges = &mut self.0;

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
    fn edge_type(&self, endpoint1: Entity, endpoint2: Entity) -> Option<EdgeType> {
        self.0
            .get(&EdgeEndpoints::new(endpoint1, endpoint2))
            .map(|e| e.0)
    }
}

#[derive(Debug, Copy, Clone, Ord, Eq, PartialOrd, PartialEq)]
struct EdgeEndpoints {
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
enum EdgeType {
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
struct PseudorandomColors {
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

fn pseudorandomize(number: u32) -> u32 {
    // inspired by legion's `U64Hasher`
    let big_prime = 2u32.pow(31) - 1; // eighth Mersenne prime, largest prime in `u32`
    big_prime.wrapping_mul(number)
}

fn block_map(
    state: &nakamoto_consensus::NakamotoNodeState,
    max_depth: usize,
) -> Vec<Vec<Option<nakamoto_consensus::Hash>>> {
    let mut main_chain = vec![];
    let mut block_hash = state.tip();
    for _ in 0..max_depth {
        if block_hash == nakamoto_consensus::Hash::default() {
            break;
        }
        main_chain.push(Some(block_hash));
        block_hash = state.hash_prev(block_hash).unwrap();
    }
    let mut result = vec![main_chain];
    let tip_height = state.height(state.tip());
    for (fork_height_diff, mut block_hash) in state
        .fork_tips()
        .iter()
        .map(|&f| (tip_height - state.height(f), f))
        .filter(|(height_diff, _)| *height_diff < max_depth)
    {
        result.push(vec![None; fork_height_diff]);
        for _ in fork_height_diff..max_depth {
            if block_hash == nakamoto_consensus::Hash::default() {
                break;
            }
            result.last_mut().unwrap().push(Some(block_hash));
            block_hash = state.hash_prev(block_hash).unwrap();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn rebuild_edges_builds_edges() {
        let mut world = World::default();
        let mut edges = EdgeMap::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet(vec![node1].into_iter().collect()),
            UnderlayPosition::new(13., 13.),
        ));

        edges.rebuild(&world);

        assert!(edges.0.contains_key(&EdgeEndpoints::new(node1, node2)));
    }

    #[wasm_bindgen_test]
    fn rebuild_edges_sets_direction() {
        let mut world = World::default();
        let mut edges = EdgeMap::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet(vec![node1].into_iter().collect()),
            UnderlayPosition::new(13., 13.),
        ));

        edges.rebuild(&world);

        assert_ne!(
            EdgeType::Undirected,
            edges
                .0
                .get(&EdgeEndpoints::new(node1, node2))
                .unwrap()
                .0
        );
    }

    #[wasm_bindgen_test]
    fn rebuild_edges_stores_removed_edges_as_phantom_edges() {
        let mut world = World::default();
        let mut edges = EdgeMap::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet(vec![node1].into_iter().collect()),
            UnderlayPosition::new(13., 13.),
        ));

        edges.rebuild(&world);

        world
            .query_one_mut::<&mut PeerSet>(node2)
            .unwrap()
            .0
            .remove(&node1);

        edges.rebuild(&world);

        assert_eq!(
            EdgeType::Phantom,
            edges
                .0
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