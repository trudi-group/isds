use super::*;
use blockchain_types::{BlockContents, Transaction};
use common::PseudorandomColors;
use nakamoto_consensus::NakamotoNodeState;

pub struct BlockchainView {
    sim: SharedSimulation,
    highlight: Highlight,
    colors: PseudorandomColors,
    cache: Cache,
    _context_handle: yew::context::ContextHandle<IsdsContext>,
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Rendered(RealSeconds),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub viewing_node: Option<Entity>,
    #[prop_or(5)]
    pub max_visible_blocks: usize,
    #[prop_or(true)]
    pub show_unconfirmed_txes: bool,
    #[prop_or(true)]
    pub show_header: bool,

    #[prop_or_default()]
    pub highlight_class: Classes,

    #[prop_or(50.)]
    pub inter_block_space: f32,
    #[prop_or(100.)]
    pub block_size: f32,
    #[prop_or(0.375)] // golden ratio
    pub block_link_relative_y: f32,
    #[prop_or(12.)]
    pub font_size: f32,
    #[prop_or(4.)]
    pub stroke_width: f32,
}

/// For checking if the node state changed so we need to rebuild the view
#[derive(Debug, Default)]
struct Cache {
    blockchain_tip: Option<Entity>,
    n_tx_unconfirmed: usize,
}

impl Component for BlockchainView {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context_data, _context_handle) = get_isds_context!(ctx, Self);

        let IsdsContext { sim, highlight, .. } = context_data;

        // TODO as props!
        let seed_palette = common::DEFAULT_SEED_PALETTE;
        let target_palette_n = 64;

        let colors = PseudorandomColors::new(seed_palette, target_palette_n);

        Self {
            sim,
            highlight,
            colors,
            cache: Default::default(), // will be set on first render
            _context_handle,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let &Props {
            viewing_node,
            max_visible_blocks,
            show_unconfirmed_txes,
            show_header,
            inter_block_space,
            block_size,
            stroke_width,
            ..
        } = ctx.props();

        let sim = self.sim.borrow();
        let state = get_node_state(viewing_node, &sim);

        let n_slots = if show_unconfirmed_txes {
            max_visible_blocks + 1
        } else {
            max_visible_blocks
        };

        html! {
            <div class="is-unselectable">
                if show_header {
                    { "The longest chain, as seen by node" }
                    <EntityName entity={ viewing_node } class="ml-2 is-family-code is-underlined" />
                }
                <svg
                   viewBox={ format!("{} {} {} {}",
                       -inter_block_space,
                       -0.5 * inter_block_space,
                       (n_slots as f32) * (block_size + inter_block_space) + inter_block_space,
                       block_size + 0.5 * inter_block_space + stroke_width,
                    ) }
                >
                    {
                        if let Some(state) = state {
                            self.view_blocks_and_unconfirmed_transactions(&state, &sim, ctx)
                        } else {
                            html! { <text x=0 y=0>{ "No node (state), no blockchain..." }</text> }
                        }
                    }
                </svg>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(_) => {
                let sim = self.sim.borrow();
                let state = get_node_state(ctx.props().viewing_node, &sim);

                // only recalculate the view if the blockchain tip or the highlight changed
                let tip_changed = if let Some(state) = state {
                    self.cache.update(&state)
                } else {
                    false
                };
                let hightlight_changed = self.highlight.update();
                tip_changed || hightlight_changed
            }
        }
    }
}

impl BlockchainView {
    fn view_blocks_and_unconfirmed_transactions(
        &self,
        state: &NakamotoNodeState,
        sim: &Simulation,
        ctx: &Context<Self>,
    ) -> Html {
        let &Props {
            max_visible_blocks,
            show_unconfirmed_txes,
            ..
        } = ctx.props();

        let blocks = last_blocks_in_longest_chain(state, max_visible_blocks, sim);
        let n_blocks = blocks.len();

        html! {
            <>
                { self.view_blocks(blocks, ctx) }
                if show_unconfirmed_txes {
                    { self.view_unconfirmed_transactions(n_blocks, state, sim, ctx) }
                }
            </>
        }
    }
    #[allow(clippy::type_complexity)] // TODO
    fn view_blocks(
        &self,
        blocks: Vec<(Option<Entity>, Vec<(Option<Entity>, String)>)>,
        ctx: &Context<Self>,
    ) -> Html {
        let Props {
            inter_block_space,
            block_size,
            ..
        } = ctx.props();
        blocks
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, (block_id, txes))| {
                let block_x = (i as f32) * (block_size + inter_block_space);

                if let Some(block_id) = block_id {
                    self.view_block(block_x, block_id, txes, ctx)
                } else {
                    self.view_genesis_block(block_x, ctx)
                }
            })
            .collect::<Html>()
    }
    fn view_unconfirmed_transactions(
        &self,
        n_blocks: usize,
        state: &NakamotoNodeState,
        sim: &Simulation,
        ctx: &Context<Self>,
    ) -> Html {
        let Props {
            inter_block_space,
            block_size,
            font_size,
            ..
        } = ctx.props();
        let base_x = (n_blocks as f32) * (block_size + inter_block_space);
        let transactions_shortform = get_unconfirmed_transactions_shortform(state, sim);
        html! {
            <g>
                if !transactions_shortform.is_empty() {
                    <text
                        x={ base_x.to_string() }
                        y="0"
                        font-size = { font_size.to_string() }
                        font-family="monospace"
                        fill="black">
                        { "Unconfirmed:" }
                    </text>
                }
                { self.view_block_text(base_x, transactions_shortform, "black".to_string(), ctx) }
            </g>
        }
    }
    fn view_block(
        &self,
        block_x: f32,
        block_id: Entity,
        transactions_shortform: Vec<(Option<Entity>, String)>,
        ctx: &Context<Self>,
    ) -> Html {
        let Props {
            block_size,
            stroke_width,
            ..
        } = ctx.props();
        let color = self.colors.get(block_id.id()).to_string();

        html! {
            <g>
                <rect
                    x={ block_x.to_string() }
                    y="0"
                    width={ block_size.to_string() }
                    height={ block_size.to_string() }
                    fill="none"
                    stroke={ color.clone() }
                    stroke-width={ stroke_width.to_string() }
                />
                { self.view_link(block_x, color.clone(), ctx) }
                { self.view_block_text(block_x, transactions_shortform, "black".to_string(), ctx) }
            </g>
        }
    }
    fn view_genesis_block(&self, block_x: f32, ctx: &Context<Self>) -> Html {
        let Props {
            block_size,
            stroke_width,
            ..
        } = ctx.props();
        html! {
            <g>
                <rect
                    x={ block_x.to_string() }
                    y="0"
                    width={ block_size.to_string() }
                    height={ block_size.to_string() }
                    fill="none"
                    stroke="gray"
                    stroke-width={ stroke_width.to_string() }
                    stroke-dasharray={ format!("{}, {}", stroke_width, stroke_width) }
                />
                {
                    self.view_block_text(
                        block_x,
                        vec![(None, "Genesis".to_string()),
                        (None, "block".to_string())],
                        "gray".to_string(),
                        ctx
                    )
                }
            </g>
        }
    }
    fn view_link(&self, block_x: f32, color: String, ctx: &Context<Self>) -> Html {
        let Props {
            block_size,
            stroke_width,
            inter_block_space,
            block_link_relative_y,
            ..
        } = ctx.props();

        let x1 = block_x;
        let y1 = block_size * block_link_relative_y;
        let x2 = x1 - (0.8 * inter_block_space);
        let y2 = y1;

        html! {
            <g>
                <line
                    x1={ x1.to_string() }
                    y1={ y1.to_string() }
                    x2={ x2.to_string() }
                    y2={ y2.to_string() }
                    stroke={ color.clone() }
                    stroke-width={ stroke_width.to_string() }
                />
                <polygon
                    points={ format!("{} {}, {} {}, {} {}",
                        x2, y2,
                        x2 + stroke_width * 2., y2 - stroke_width * 2.,
                        x2 + stroke_width * 2., y2 + stroke_width * 2.,
                    )}
                    stroke-width={ stroke_width.to_string() }
                    stroke={ color.clone() }
                    fill={ color.clone() }
                />
            </g>
        }
    }
    fn view_block_text(
        &self,
        block_x: f32,
        lines_with_associated_entities: Vec<(Option<Entity>, String)>,
        color: String,
        ctx: &Context<Self>,
    ) -> Html {
        let Props {
            highlight_class: tx_highlight_class,
            block_size,
            stroke_width,
            font_size,
            ..
        } = ctx.props();
        let hl = &self.highlight;

        let text_x = block_x + 2. * stroke_width;
        let text_y = 2. * stroke_width;

        let max_lines = ((block_size - 4. * stroke_width) / (font_size * 1.2)).floor() as usize;
        let max_line_len = ((block_size - 4. * stroke_width) / (font_size * 0.6)).floor() as usize;
        let lines = maybe_truncate_lines(lines_with_associated_entities, max_lines, max_line_len);

        html! {
            <text
                x={ text_x.to_string() }
                y={ text_y.to_string() }
                font-size = { font_size.to_string() }
                font-family="monospace"
                fill={ color }>
                {
                    lines.into_iter().enumerate().map(|(i, (entity, line))| {

                        let (onmouseover, onmouseout, onclick) = if let Some(e) = entity {
                            (
                                hl.set_hover_callback(e),
                                hl.reset_hover_callback(),
                                hl.toggle_select_callback(e)
                            )
                            } else {
                            (Callback::noop(), Callback::noop(), Callback::noop())
                        };

                        html! {
                            <tspan
                                x={ text_x.to_string() }
                                dy={ if i == 0 { "1em" } else { "1.2em" } }
                                class={
                                    classes!(
                                        entity.map(|_| "is-clickable"),
                                        entity.and_then(
                                            |e| self.highlight.is(e).then_some(
                                                tx_highlight_class.clone()
                                            )
                                        ),
                                    )
                                }
                                { onmouseover }
                                { onmouseout }
                                { onclick }
                            >
                                { line }
                            </tspan>
                        }
                    }).collect::<Html>()
                }
            </text>
        }
    }
}

impl Cache {
    fn update(&mut self, state: &NakamotoNodeState) -> bool {
        if state.tip() != self.blockchain_tip
            || self.n_tx_unconfirmed != state.txes_unconfirmed().len()
        {
            self.blockchain_tip = state.tip();
            self.n_tx_unconfirmed = state.txes_unconfirmed().len();
            true
        } else {
            false
        }
    }
}

#[allow(clippy::type_complexity)] // TODO
fn last_blocks_in_longest_chain(
    state: &nakamoto_consensus::NakamotoNodeState,
    max_blocks: usize,
    sim: &Simulation,
) -> Vec<(Option<Entity>, Vec<(Option<Entity>, String)>)> {
    let mut chain = vec![];
    let mut block_id = state.tip();
    for _ in 0..max_blocks {
        // The block with id `None` will be counted as the genesis block
        chain.push((block_id, get_transactions_shortform(block_id, sim)));
        if block_id == None {
            break;
        }
        block_id = state.block_header(block_id.unwrap()).unwrap().id_prev;
    }
    chain
}

fn get_node_state(
    node_id: Option<Entity>,
    sim: &Simulation,
) -> Option<hecs::Ref<NakamotoNodeState>> {
    node_id.and_then(|node_id| sim.world.get::<NakamotoNodeState>(node_id).ok())
}

fn get_transactions_shortform(
    block_id: Option<Entity>,
    sim: &Simulation,
) -> Vec<(Option<Entity>, String)> {
    if let Some(block_id) = block_id {
        let block_contents = sim.world.get::<BlockContents>(block_id).unwrap();
        block_contents
            .iter()
            .map(|&txid| (txid, sim.world.get::<Transaction>(txid).unwrap()))
            .map(|(txid, tx)| (Some(txid), transaction_shortform(&tx)))
            .collect()
    } else {
        vec![]
    }
}

fn get_unconfirmed_transactions_shortform(
    state: &NakamotoNodeState,
    sim: &Simulation,
) -> Vec<(Option<Entity>, String)> {
    state
        .txes_unconfirmed()
        .iter()
        .map(|&txid| (txid, sim.world.get::<Transaction>(txid).unwrap()))
        .map(|(txid, tx)| (Some(txid), transaction_shortform(&tx)))
        .collect()
}
fn transaction_shortform(tx: &Transaction) -> String {
    format!(
        "[{}->{}: {}]",
        tx.from.chars().next().unwrap_or('_'),
        tx.to.chars().next().unwrap_or('_'),
        blockchain_types::coins_from(tx.value as i64),
    )
}

fn maybe_truncate_lines(
    mut lines: Vec<(Option<Entity>, String)>,
    max_lines: usize,
    max_line_len: usize,
) -> Vec<(Option<Entity>, String)> {
    if lines.len() > max_lines {
        lines.truncate(max_lines.saturating_sub(1));
        lines.push((None, "...".to_string()));
    }
    lines
        .into_iter()
        .map(|(entity, line)| (entity, maybe_truncate_line(line, max_line_len)))
        .collect()
}

fn maybe_truncate_line(s: String, max_len: usize) -> String {
    if s.len() > max_len {
        if s.len() < 3 {
            s[..max_len].to_string()
        } else if s.len() < 8 {
            format!("{}..", &s[..(max_len - 2)])
        } else {
            format!("{}..{}", &s[..(max_len - 4)], &s[(s.len() - 2)..])
        }
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn truncate_very_short_string() {
        let s = "he".to_string();
        let expected = "h".to_string();
        let actual = maybe_truncate_line(s, 1);
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn truncate_short_string() {
        let s = "hehehe".to_string();
        let expected = "heh..".to_string();
        let actual = maybe_truncate_line(s, 5);
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn truncate_typical_string() {
        let s = "0123456789".to_string();
        let expected = "01234..89".to_string();
        let actual = maybe_truncate_line(s, 9);
        assert_eq!(expected, actual);
    }
}
