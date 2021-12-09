use super::*;
use view_helpers::*;

// TODO: handle key events in top-level element!

impl Isds {
    pub fn view_menu_bar(&self, ctx: &Context<Isds>) -> Html {
        let link = ctx.link();
        html!{
            <div style="max-width: 1024px">
                <button onclick={ link.callback(|_| Msg::UserPausePlay) }>
                    if self.sim.time.paused() {
                        { "▶️" }
                    } else {
                        { "⏸️" }
                    }
                </button>
                <button onclick={ link.callback(|_| Msg::UserMakeSlower) }>
                    { "⏪" }
                </button>
                <button onclick={ link.callback(|_| Msg::UserMakeFaster) }>
                    { "⏩" }
                </button>
                { format!(
                    " | Sim time (s): {:.3} ({}✕)",
                    self.sim.time.now(),
                    self.sim.time.speed() as f32 // downcasting makes it look nicer when printed
                ) }
                if self.show_debug_infos {
                    { format!(" | FPS: {:.0}", self.fps.get()) }
                }
                <span
                    style=r#"float: "right"; margin-top: 5px; margin-bottom: 5px, cursor: "pointer""#
                    onclick={ link.callback(|_| Msg::UserToggleHelp) }
                >
                    { "[?]" }
                </span>
            </div>
        }
    }
}

pub fn view_palette(view_cache: &ViewCache) -> Html {
    view_cache
        .colors()
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

pub fn view_nodes(world: &World, view_cache: &ViewCache, ctx: &Context<Isds>) -> Html {
    let r = 5.0;
    let link = ctx.link();
    world
        .query::<(&UnderlayPosition, &nakamoto_consensus::NakamotoNodeState)>()
        .into_iter()
        .map(|(node, (pos, state))| html! {
            <g>
                <circle
                    cx={ pos.x.to_string() }
                    cy={ pos.y.to_string() }
                    r={ r.to_string() }
                    onclick={ link.callback(move |_| Msg::NodeClick(node)) }
                />
                { view_blocks(view_cache, state, pos.x + 8., pos.y - 8.) }
            </g>
        })
        .collect()
}

pub fn view_edges(view_cache: &ViewCache, ctx: &Context<Isds>) -> Html {
    let link = ctx.link();
    view_cache
        .edges()
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

pub fn view_messages(world: &World, view_cache: &ViewCache, time_now: SimSeconds) -> Html {
    world
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
                    fill={ view_cache.color(nakamoto_consensus::to_number(block.hash())).to_string() }
                />
            }
        }).collect()
}

pub fn view_log<'a>(
    message_log: impl DoubleEndedIterator<Item = &'a (SimSeconds, String)>,
) -> Html {
    html! {
        <pre>
            {
                message_log
                    .rev()
                    .map(|(time, message)| format!("{:.3}: {}\n", time, message)).collect::<Html>()
            }
        </pre>
    }
}

// pub fn view_help(show_help: bool) -> Node<Msg> {
//     let help_message = md!(indoc! {r#"
//         # Interactive Simulation of Nakamoto Consensus

//         Shows how ₿itcoin works. Roughly.

//         *Work in progress!*

//         ## Getting started

//         Nodes find new blocks when you click on them.

//         Links between nodes disappear and reappear when you click on them.

//         Try partitioning the network to create a fork!
//         Then have a look at what happens when you reconnect the partitions.

//         ## Some handy keyboard shortcuts

//         - `[space]` ⇨ pause/play simulation
//         - `[←]`/`[→]`, `[h]`/`[l]` ⇨ control simulation speed
//         - `[m]` ⇨ a random node will "mine" a block
//         - `[?]` ⇨ show this page

//         ## Where is the code?

//         [Here](https://github.com/wiberlin/isds-bitcoin-prototype).

//         ## Feedback is very welcome!

//         -- [Martin](https://www.weizenbaum-institut.de/en/portrait/p/martin-florian/)
//     "#});

//     div![
//         style![
//             St::Display => if show_help { "block" } else { "none" },
//             St::Position => "fixed",
//             St::ZIndex => 1,
//             St::Padding => px(5),
//             St::PaddingTop => px(80),
//             St::Top =>  px(0),
//             St::Left =>  px(0),
//             St::Right =>  px(0),
//             St::Height => percent(100),
//             St::BackgroundColor => "rgba(0,0,0,0.5)",
//         ],
//         div![
//             style![
//                 St::Padding => px(20),
//                 St::Margin => "auto",
//                 St::MaxWidth => px(900),
//                 St::MaxHeight => "calc(100vh - 165px)",
//                 St::OverflowY => "auto",
//                 St::BackgroundColor => "white",
//             ],
//             span![
//                 style! {
//                     St::Float => "right",
//                     St::MarginTop => px(5),
//                     St::MarginBottom => px(5),
//                     St::Cursor => "pointer",
//                 },
//                 "[✕]",
//                 ev(Ev::Click, move |_| Msg::UserToggleHelp),
//             ],
//             help_message,
//             ev(Ev::Click, |event| {
//                 event.stop_propagation();
//             })
//         ],
//         ev(Ev::Click, move |_| Msg::UserToggleHelp),
//     ]
// }

pub fn view_blocks(
    view_cache: &ViewCache,
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
                                stroke={ (view_cache.color(nakamoto_consensus::to_number(block_map[i][j-1].unwrap()))).to_string() }
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
                            fill={ view_cache.color(nakamoto_consensus::to_number(block_hash)).to_string() }
                        />
                    });
                    result.push(html! {
                        <line
                            x1={ (x + (block_width + block_spacing) * (i as f32) + block_width / 2.).to_string() }
                            x2={ (x + (block_width + block_spacing) * (i as f32) + block_width / 2.).to_string() }
                            y1={ (y + (block_height + block_spacing) * (j as f32) + block_height).to_string() }
                            y2={ (y + (block_height + block_spacing) * ((j + 1) as f32)).to_string() }
                            stroke={ (view_cache.color(nakamoto_consensus::to_number(block_hash))).to_string() }
                        />
                    });
                }
            }
        }
    }
    result.into_iter().collect()
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
