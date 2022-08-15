use super::*;
use common::PseudorandomColors;

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

pub struct NetView {
    sim: SharedSimulation,
    highlight: Highlight,
    colors: PseudorandomColors,
    edges: EdgeMap,
    _context_handle: yew::context::ContextHandle<IsdsContext>,
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Rendered(RealSeconds),
    NodeClick(Entity),
    NodeMouseOver(Entity),
    NodeMouseOut,
    LinkClick(Entity, Entity),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default()]
    pub on_node_click: Option<Callback<Entity>>,

    #[prop_or(false)]
    pub node_highlight_on_hover: bool,

    #[prop_or(true)]
    pub toggle_edges_on_click: bool,

    #[prop_or_default()]
    pub highlight_class: Classes,

    #[prop_or(50.)]
    pub buffer_space: f32,
    // TODO a lot more things should be props really
}

impl Component for NetView {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context_data, _context_handle) = get_isds_context!(ctx, Self);

        let IsdsContext { sim, highlight, .. } = context_data;

        // TODO as props!
        let seed_palette = common::DEFAULT_SEED_PALETTE;
        let target_palette_n = 64;

        let colors = PseudorandomColors::new(seed_palette, target_palette_n);
        let edges = EdgeMap::new(&sim.borrow().world, sim.borrow().time.now());

        Self {
            sim,
            highlight,
            colors,
            edges,
            _context_handle,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let buffer_space = ctx.props().buffer_space;
        html! {
            <>
                <style>
                    { " .phantom-link { opacity: 0.0; } .phantom-link:hover { opacity: 1.0; }" }
                </style>
                <svg
                    class={ "is-unselectable" } // for avoiding accidental selects on Chrome
                    viewBox={ format!("{} {} {} {}",
                       -buffer_space,
                       -buffer_space,
                       self.sim.borrow().underlay_width() + 2. * buffer_space,
                       self.sim.borrow().underlay_height() + 2. * buffer_space
                    ) }
                >
                    // { self.view_palette() }
                    { self.view_edges(ctx) }
                    { self.view_nodes(ctx) }
                    { self.view_messages(ctx) }
                </svg>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(_) => {
                self.rebuild_edges_if_changed();
                true // often enough, we'll have in-flight messages that have to be redrawn
            }
            Msg::NodeClick(node) => {
                log!(format!("Click on {}", self.sim.borrow().name(node)));
                if let Some(on_node_click) = ctx.props().on_node_click.as_ref() {
                    on_node_click.emit(node);
                } else if ctx.props().node_highlight_on_hover {
                    self.highlight.toggle_select(node);
                }
                false
            }
            Msg::NodeMouseOver(node) => {
                if ctx.props().node_highlight_on_hover {
                    self.highlight.set_hover(node);
                    true
                } else {
                    false
                }
            }
            Msg::NodeMouseOut => {
                if ctx.props().node_highlight_on_hover {
                    self.highlight.reset_hover();
                    true
                } else {
                    false
                }
            }
            Msg::LinkClick(node1, node2) => {
                if ctx.props().toggle_edges_on_click {
                    // TODO perhaps configure link click action using a property?
                    log!(format!(
                        "Click on link between {} and {}.",
                        self.sim.borrow().name(node1),
                        self.sim.borrow().name(node2)
                    ));
                    if self.edges.edge_type(node1, node2).unwrap().is_phantom() {
                        self.sim.borrow_mut().do_now(AddPeer(node1, node2));
                        self.sim.borrow_mut().do_now(AddPeer(node2, node1));
                    } else {
                        self.sim.borrow_mut().do_now(RemovePeer(node1, node2));
                        self.sim.borrow_mut().do_now(RemovePeer(node2, node1));
                    }
                }
                false
            }
        }
    }
}

impl NetView {
    fn rebuild_edges_if_changed(&mut self) -> bool {
        let now = self.sim.borrow().time.now();
        self.edges.rebuild_if_needed(&self.sim.borrow().world, now)
    }
    fn view_nodes(&self, ctx: &Context<NetView>) -> Html {
        let r = 5.0;
        let link = ctx.link();
        self.sim
            .borrow()
            .world
            .query::<(&UnderlayPosition, &nakamoto_consensus::NakamotoNodeState)>()
            .into_iter()
            .map(|(node, (pos, node_state))| {
                html! {
                    <g>
                        <circle
                            class={
                                classes!(
                                    self.highlight
                                        .is(node)
                                        .then_some(ctx.props().highlight_class.clone()),
                                        (
                                            ctx.props().on_node_click.is_some() ||
                                            ctx.props().node_highlight_on_hover
                                        ).then_some("is-clickable"),
                                )
                            }
                            cx={ pos.x.to_string() }
                            cy={ pos.y.to_string() }
                            r={ r.to_string() }
                            onclick={ link.callback(move |_| Msg::NodeClick(node)) }
                            onmouseover={ link.callback(move |_| Msg::NodeMouseOver(node)) }
                            onmouseout={ link.callback(|_| Msg::NodeMouseOut) }
                        />
                        { self.view_blocks(node_state, pos.x + 8., pos.y - 8.) }
                    </g>
                }
            })
            .collect()
    }
    fn view_edges(&self, ctx: &Context<NetView>) -> Html {
        let link = ctx.link();
        self.edges
            .edges
            .iter()
            .map(|(&edge_endpoints, &(edge_type, line))| {
                html! {
                    <g
                        onclick={ link.callback(move |_| Msg::LinkClick(
                            edge_endpoints.left(),
                            edge_endpoints.right()
                        )) }
                    >
                        if ctx.props().toggle_edges_on_click {
                            <line
                                class={ classes!("phantom-link", "is-clickable") }
                                x1={ line.start.x.to_string() }
                                y1={ line.start.y.to_string() }
                                x2={ line.end.x.to_string() }
                                y2={ line.end.y.to_string() }
                                stroke="gray"
                                stroke-opacity="0.3"
                                stroke-width=8
                            />
                        }
                        if edge_type != EdgeType::Phantom {
                            if edge_type == EdgeType::Undirected {
                                <line
                                    x1={ line.start.x.to_string() }
                                    y1={ line.start.y.to_string() }
                                    x2={ line.end.x.to_string() }
                                    y2={ line.end.y.to_string() }
                                    stroke="gray"
                                    class={
                                        classes!(
                                            ctx.props().toggle_edges_on_click.then_some("is-clickable")
                                        )
                                    }
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
                                    class={
                                        classes!(
                                            ctx.props().toggle_edges_on_click.then_some("is-clickable")
                                        )
                                    }
                                />
                            }
                        }
                    </g>
                }
            })
            .collect()
    }
    fn view_messages(&self, ctx: &Context<NetView>) -> Html {
        // TODO: currently more like: view_nakamoto_consensus_messages...
        let time_now = self.sim.borrow().time.now();
        self.sim
            .borrow()
            .world
            .query::<(
                &UnderlayLine,
                &TimeSpan,
                &simple_flooding::SimpleFloodingMessage<nakamoto_consensus::InventoryItem>,
            )>()
            .into_iter()
            .map(|(_, (trajectory, time_span, message))| {
                let (x, y) = message_position(trajectory, time_span, time_now);
                match message.0 {
                    nakamoto_consensus::InventoryItem::Transaction(txid) => {
                        html! {
                            <circle
                                class={
                                    classes!(
                                        self.highlight
                                            .is(txid)
                                            .then_some(ctx.props().highlight_class.clone()),
                                    )
                                }
                                cx={ x.to_string() }
                                cy={ y.to_string() }
                                r=1.5
                            />
                        }
                    }
                    nakamoto_consensus::InventoryItem::Block(block_id) => {
                        html! {
                            <circle
                                cx={ x.to_string() }
                                cy={ y.to_string() }
                                r=2
                                fill={ self.colors.get(block_id.id()).to_string() }
                            />
                        }
                    }
                }
            })
            .collect()
    }
    fn view_blocks(&self, state: &nakamoto_consensus::NakamotoNodeState, x: f32, y: f32) -> Html {
        let max_depth = 5;
        let block_height = 5.;
        let block_width = 5.;
        let block_spacing = 2.;

        let block_map = blocks_cutout(state, max_depth);
        let mut result = vec![];

        for i in 0..block_map.len() {
            for j in 0..block_map[i].len() {
                if let Some(block_id) = block_map[i][j] {
                    if let Some(k) = block_map.iter().take(i).enumerate().find_map(|(k, chain)| {
                        if let Some(other_chain_id) = chain[j] {
                            if other_chain_id == block_id {
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
                                    stroke={ self.colors.get(block_map[i][j-1].unwrap().id()).to_string() }
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
                                fill={ self.colors.get(block_id.id()).to_string() }
                            />
                        });
                        result.push(html! {
                            <line
                                x1={ (x + (block_width + block_spacing) * (i as f32) + block_width / 2.).to_string() }
                                x2={ (x + (block_width + block_spacing) * (i as f32) + block_width / 2.).to_string() }
                                y1={ (y + (block_height + block_spacing) * (j as f32) + block_height).to_string() }
                                y2={ (y + (block_height + block_spacing) * ((j + 1) as f32)).to_string() }
                                stroke={ (self.colors.get(block_id.id())).to_string() }
                            />
                        });
                    }
                }
            }
        }
        result.into_iter().collect()
    }
    // TODO: this should be separate component; s.a. palette as prop todo above
    pub fn view_palette(&self) -> Html {
        self.colors
            .all()
            .iter()
            .enumerate()
            .map(|(i, color)| {
                html! {
                <circle
                    cx={ format!("{}", -40 + 5 * (i as i32)) }
                    cy={ format!("{}", -40) }
                    r=5
                    fill={ color.clone() }
                />
                }
            })
            .collect()
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
struct EdgeMap {
    edges: BTreeMap<EdgeEndpoints, (EdgeType, UnderlayLine)>,
    last_update: SimSeconds,
}
impl EdgeMap {
    fn new(world: &World, simtime_now: SimSeconds) -> Self {
        let mut new: Self = Default::default();
        new.rebuild(world, simtime_now);
        new
    }

    fn rebuild_if_needed(&mut self, world: &World, simtime_now: SimSeconds) -> bool {
        if self.needs_rebuild(world) {
            self.rebuild(world, simtime_now);
            true
        } else {
            false
        }
    }

    fn needs_rebuild(&self, world: &World) -> bool {
        world
            .query::<&PeerSet>()
            .iter()
            .any(|(_, peer_set)| peer_set.last_update() > self.last_update)
    }

    fn rebuild(&mut self, world: &World, simtime_now: SimSeconds) {
        let edges = &mut self.edges;

        for (edge_type, _) in edges.values_mut() {
            *edge_type = EdgeType::Phantom;
        }

        log!("Rebuilding edges...");

        for (node, peer_set) in world.query::<&PeerSet>().iter() {
            for &peer in peer_set.iter() {
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
        self.last_update = simtime_now;
    }
    fn edge_type(&self, endpoint1: Entity, endpoint2: Entity) -> Option<EdgeType> {
        self.edges
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

fn blocks_cutout(
    state: &nakamoto_consensus::NakamotoNodeState,
    max_depth: usize,
) -> Vec<Vec<Option<Entity>>> {
    let mut main_chain = vec![];
    let mut block_id = state.tip();
    for _ in 0..max_depth {
        if block_id == None {
            break;
        }
        main_chain.push(block_id);
        block_id = state.block_header(block_id.unwrap()).unwrap().id_prev;
    }
    let mut result = vec![main_chain];
    for (fork_height_diff, mut block_id) in state
        .fork_tips()
        .iter()
        .map(|&ft| {
            (
                state.tip_height() - state.block_header(ft).unwrap().height,
                Some(ft),
            )
        })
        .filter(|(height_diff, _)| *height_diff < max_depth)
    {
        result.push(vec![None; fork_height_diff]);
        for _ in fork_height_diff..max_depth {
            if block_id == None {
                break;
            }
            result.last_mut().unwrap().push(block_id);
            block_id = state.block_header(block_id.unwrap()).unwrap().id_prev;
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
    fn rebuild_builds_edges() {
        let mut world = World::default();
        let mut edges = EdgeMap::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet::default_from(vec![node1]),
            UnderlayPosition::new(13., 13.),
        ));

        edges.rebuild(&world, Default::default());

        assert!(edges.edges.contains_key(&EdgeEndpoints::new(node1, node2)));
    }

    #[wasm_bindgen_test]
    fn rebuild_sets_direction() {
        let mut world = World::default();
        let mut edges = EdgeMap::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet::default_from(vec![node1]),
            UnderlayPosition::new(13., 13.),
        ));

        edges.rebuild(&world, Default::default());

        assert_ne!(
            EdgeType::Undirected,
            edges
                .edges
                .get(&EdgeEndpoints::new(node1, node2))
                .unwrap()
                .0
        );
    }

    #[wasm_bindgen_test]
    fn rebuild_stores_removed_edges_as_phantom_edges() {
        let mut world = World::default();
        let mut edges = EdgeMap::default();
        let node1 = world.spawn((PeerSet::default(), UnderlayPosition::new(23., 42.)));
        let node2 = world.spawn((
            PeerSet::default_from(vec![node1]),
            UnderlayPosition::new(13., 13.),
        ));

        edges.rebuild(&world, Default::default());

        world
            .query_one_mut::<&mut PeerSet>(node2)
            .unwrap()
            .remove(&node1, Default::default());

        edges.rebuild(&world, Default::default());

        assert_eq!(
            EdgeType::Phantom,
            edges
                .edges
                .get(&EdgeEndpoints::new(node1, node2))
                .unwrap()
                .0,
        );
    }
}
