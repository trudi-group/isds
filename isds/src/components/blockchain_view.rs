use super::*;
use blockchain_types::{BlockContents, Transaction};
use common::PseudorandomColors;
use nakamoto_consensus::NakamotoNodeState;

pub struct BlockchainView {
    sim: SharedSimulation,
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
    #[prop_or(5)]
    pub max_visible_blocks: usize,
    #[prop_or_default]
    pub viewing_node: Option<Entity>,

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

        let sim = context_data.sim;

        // TODO as props!
        let seed_palette = common::DEFAULT_SEED_PALETTE;
        let target_palette_n = 64;

        let colors = PseudorandomColors::new(seed_palette, target_palette_n);

        Self {
            sim,
            colors,
            cache: Default::default(), // will be set on first render
            _context_handle,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Props {
            viewing_node,
            inter_block_space,
            block_size,
            max_visible_blocks,
            ..
        } = ctx.props();

        let sim = self.sim.borrow();
        let state = get_node_state(*viewing_node, &sim);

        html! {
            <>
                { "View on main chain of node" }
                <span class="ml-2 is-family-code">
                    { viewing_node.map_or("None".to_string(), |id| self.sim.borrow().name(id)) }
                </span>
                <svg
                   viewBox={ format!("{} {} {} {}",
                       -inter_block_space,
                       -inter_block_space,
                       ((max_visible_blocks + 1) as f32) * (block_size + inter_block_space) + inter_block_space,
                       block_size + 2. * inter_block_space,
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
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(_) => {
                let sim = self.sim.borrow();
                let state = get_node_state(ctx.props().viewing_node, &sim);

                // only recalculate the view if the blockchain tip changed
                if let Some(state) = state {
                    self.cache.update(&state)
                } else {
                    false
                }
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
        let Props {
            max_visible_blocks, ..
        } = ctx.props();

        let blocks = last_blocks_in_longest_chain(state, *max_visible_blocks, sim);
        let n_blocks = blocks.len();

        html! {
            <>
                { self.view_blocks(blocks, ctx) }
                { self.view_unconfirmed_transactions(n_blocks, state, sim, ctx) }
            </>
        }
    }
    fn view_blocks(&self, blocks: Vec<(Option<Entity>, Vec<String>)>, ctx: &Context<Self>) -> Html {
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
        transactions_shortform: Vec<String>,
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
                        vec!["Genesis".to_string(),
                        "block and this is a very ver very very very long string".to_string()],
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
        lines: Vec<String>,
        color: String,
        ctx: &Context<Self>,
    ) -> Html {
        let Props {
            block_size,
            stroke_width,
            font_size,
            ..
        } = ctx.props();
        let text_x = block_x + 2. * stroke_width;
        let text_y = 2. * stroke_width;

        let max_lines = ((block_size - 4. * stroke_width) / (font_size * 1.2)).floor() as usize;
        let max_line_len = ((block_size - 4. * stroke_width) / (font_size * 0.6)).floor() as usize;
        let lines = maybe_truncate_lines(lines, max_lines, max_line_len);

        html! {
            <text
                x={ text_x.to_string() }
                y={ text_y.to_string() }
                font-size = { font_size.to_string() }
                font-family="monospace"
                fill={ color }>
                {
                    lines.into_iter().enumerate().map(|(i, line)| html! {
                        <tspan
                            x={ text_x.to_string() }
                            dy={ if i == 0 { "1em" } else { "1.2em" } }>
                            { line }
                        </tspan>
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

fn last_blocks_in_longest_chain(
    state: &nakamoto_consensus::NakamotoNodeState,
    max_blocks: usize,
    sim: &Simulation,
) -> Vec<(Option<Entity>, Vec<String>)> {
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

fn get_transactions_shortform(block_id: Option<Entity>, sim: &Simulation) -> Vec<String> {
    if let Some(block_id) = block_id {
        let block_contents = sim.world.get::<BlockContents>(block_id).unwrap();
        block_contents
            .iter()
            .map(|tx_id| sim.world.get::<Transaction>(*tx_id).unwrap())
            .map(|tx| transaction_shortform(&tx))
            .collect()
    } else {
        vec![]
    }
}

fn get_unconfirmed_transactions_shortform(
    state: &NakamotoNodeState,
    sim: &Simulation,
) -> Vec<String> {
    state
        .txes_unconfirmed()
        .iter()
        .map(|tx_id| sim.world.get::<Transaction>(*tx_id).unwrap())
        .map(|tx| transaction_shortform(&tx))
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
    mut lines: Vec<String>,
    max_lines: usize,
    max_line_len: usize,
) -> Vec<String> {
    if lines.len() > max_lines {
        lines.truncate(max_lines.saturating_sub(1));
        lines.push("...".to_string());
    }
    lines
        .into_iter()
        .map(|line| maybe_truncate_line(line, max_line_len))
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
