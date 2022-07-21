use super::*;

pub struct BlockchainView {
    sim: SharedSimulation,
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
}

impl Component for BlockchainView {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context_data, _context_handle) = get_isds_context!(ctx, Self);

        let sim = context_data.sim;

        Self {
            sim,
            _context_handle,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Just a mock so far
        let inter_block_space = 50.;
        let block_size = 100.;
        let block_link_relative_y = 0.375; // golden ratio
        let stroke_width = 4.;
        let max_visible_blocks = ctx.props().max_visible_blocks;
        html! {
            <svg
               viewBox={ format!("{} {} {} {}",
                   -inter_block_space,
                   -inter_block_space,
                   ((max_visible_blocks + 1) as f32) * (block_size + inter_block_space) + inter_block_space,
                   block_size + 2. * inter_block_space,
                ) }
            >
                { // view blocks
                    (0..max_visible_blocks).map(|i| {
                        let x1 = (i as f32) * (block_size + inter_block_space);
                        let y1 = block_size * block_link_relative_y;
                        let x2 = x1 - (0.8 * inter_block_space);
                        let y2 = y1;
                        html! {
                            <g>
                                <rect
                                    x={ x1.to_string() }
                                    y="0"
                                    width={ block_size.to_string() }
                                    height={ block_size.to_string() }
                                    fill="none"
                                    stroke="black"
                                    stroke-width={ stroke_width.to_string() }
                                />
                                <line
                                    x1={ x1.to_string() }
                                    x2={ x2.to_string() }
                                    y1={ y1.to_string() }
                                    y2={ y2.to_string() }
                                    stroke="black"
                                    stroke-width={ stroke_width.to_string() }
                                />
                                <polygon
                                    points={ format!("{} {}, {} {}, {} {}",
                                        x2, y2,
                                        x2 + stroke_width * 2., y2 - stroke_width * 2.,
                                        x2 + stroke_width * 2., y2 + stroke_width * 2.,
                                    )}
                                    stroke-width={ stroke_width.to_string() }
                                    stroke="black"
                                    fill="black"
                                />
                                <svg
                                    x={ (x1 + 2. * stroke_width).to_string() }
                                    y={ (2. * stroke_width).to_string() }
                                    width={ (block_size - 4. * stroke_width).to_string() }
                                    height={ (block_size - 4. * stroke_width).to_string() }
                                >
                                    {
                                        [ "[ A->B: 15 ]", "[ C->B: 1337 ]", "[ B->A: 2345 ]" ]
                                            .into_iter()
                                            .enumerate()
                                            .map(|(j, tx_string)| html! {
                                                <text
                                                    x="0"
                                                    y={ (12. * (j as f32 + 1.) + stroke_width * j as f32) .to_string() }
                                                    font-size=12
                                                >
                                                    { tx_string }
                                                </text>
                                            }).collect::<Html>()
                                    }
                                </svg>
                            </g>
                        }
                    }).collect::<Html>()
                }
            </svg>
        }
    }
}
