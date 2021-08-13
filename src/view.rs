use super::*;
use view_helpers::*;

// `view` describes what to display.
pub fn view(model: &Model) -> impl IntoNodes<Msg> {
    let sim_time = model.sim.time.now();
    let buffer_space = 50.;
    nodes![
        div![
            style! {
                St::MaxWidth => px(1024),
            },
            button![
                if model.sim.time.paused() {
                    "▶️"
                } else {
                    "⏸️"
                },
                ev(Ev::Click, |_| Msg::UserPausePlay),
            ],
            button!["⏪", ev(Ev::Click, |_| Msg::UserMakeSlower),],
            button!["⏩", ev(Ev::Click, |_| Msg::UserMakeFaster),],
            format!(
                " | Sim time (s): {:.3} ({}✕)",
                sim_time,
                model.sim.time.speed() as f32 // downcasting makes it look nicer when printed
            ),
            format!(" | FPS: {:.0}", model.fps.get()),
            span![
                style! {
                    St::Float => "right",
                    St::MarginTop => px(5),
                    St::MarginBottom => px(5),
                    St::Cursor => "pointer",
                },
                "[?]",
                ev(Ev::Click, move |_| Msg::UserToggleHelp),
            ]
        ],
        svg![
            attrs! {
                At::ViewBox => format!("{} {} {} {}",
                    -buffer_space,
                    -buffer_space,
                    model.sim.underlay_width() + 2. * buffer_space,
                    model.sim.underlay_height() + 2. * buffer_space
                ),
            },
            style! {
                St::BorderStyle => "solid",
                St::MaxWidth => px(1024),
            },
            view_palette(&model.view_cache),
            view_edges(&model.view_cache),
            view_messages(&model.sim.world, &model.view_cache, sim_time),
            view_nodes(&model.sim.world, &model.view_cache),
        ],
        view_log(model.sim.logger.entries()),
        view_help(model.show_help),
    ]
}

fn view_palette(view_cache: &ViewCache) -> Vec<Node<Msg>> {
    view_cache
        .colors()
        .iter()
        .enumerate()
        .map(|(i, color)| {
            circle![attrs! {
                At::Cx => -40. + (5 * i) as f32,
                At::Cy => -40.,
                At::R => 5.,
                At::Fill => color,
            }]
        })
        .collect()
}

fn view_nodes(world: &World, view_cache: &ViewCache) -> Vec<Node<Msg>> {
    let r = 5.0;
    world
        .query::<(&UnderlayPosition, &nakamoto_consensus::NakamotoNodeState)>()
        .into_iter()
        .map(|(node, (pos, state))| {
            g![
                circle![
                    attrs! {
                        At::Cx => pos.x,
                        At::Cy => pos.y,
                        At::R => r,
                    },
                    ev(Ev::Click, move |_| Msg::NodeClick(node)),
                ],
                view_blocks(view_cache, state, pos.x + 8., pos.y - 8.),
            ]
        })
        .collect()
}

fn view_edges(view_cache: &ViewCache) -> Vec<Node<Msg>> {
    view_cache
        .edges()
        .iter()
        .map(|(&edge_endpoints, &(edge_type, line))| {
            g![
                line_![
                    C!["phantom-link"],
                    attrs! {
                        At::X1 => line.start.x,
                        At::Y1 => line.start.y,
                        At::X2 => line.end.x,
                        At::Y2 => line.end.y,
                        At::Stroke => "yellow",
                        At::StrokeWidth => 8,
                    }
                ],
                IF!(edge_type != EdgeType::Phantom => line_![
                    attrs! {
                        At::X1 => line.start.x,
                        At::Y1 => line.start.y,
                        At::X2 => line.end.x,
                        At::Y2 => line.end.y,
                    },
                    if edge_type == EdgeType::Undirected {
                        attrs! {
                            At::Stroke => "gray",
                        }
                    } else {
                        // TODO: https://developer.mozilla.org/en-US/docs/Web/SVG/Element/marker
                        attrs! {
                            At::Stroke => "lightgray",
                            At::StrokeDashArray => "8,8",
                        }
                    },
                ]),
                ev(Ev::Click, move |_| Msg::LinkClick(
                    edge_endpoints.left(),
                    edge_endpoints.right()
                )),
            ]
        })
        .collect()
}

fn view_messages(world: &World, view_cache: &ViewCache, time_now: SimSeconds) -> Vec<Node<Msg>> {
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
            circle![attrs! {
                At::Cx => x,
                At::Cy => y,
                At::R => 2.0,
                At::Fill => view_cache.color(nakamoto_consensus::to_number(block.hash())),
            }]
        })
        .collect()
}

fn view_log<'a>(
    message_log: impl DoubleEndedIterator<Item = &'a (SimSeconds, String)>,
) -> Node<Msg> {
    pre![message_log
        .rev()
        .map(|(time, message)| { format!("{:.3}: {}\n", time, message) })]
}

fn view_help(show_help: bool) -> Node<Msg> {
    // FIXME Markdown rendering
    let help_message = md![r#"
    *Work in progress!*

    Nodes find new blocks when you click on them.

    Some handy keyboard shortcuts:

    - `[space]` => pause/play simulation
    - `[←]`/`[→]`, `[h]`/`[l]` => control simulation speed
    - `[m]` => a random node will "mine" a block
    - `[?]` => show this page
    "#];

    div![
        style![
            St::Display => if show_help { "block" } else { "none" },
            St::Position => "fixed",
            St::ZIndex => 1,
            St::PaddingTop => px(100),
            St::Top =>  px(0),
            St::Left =>  px(0),
            St::Right =>  px(0),
            St::Height => percent(100),
            St::BackgroundColor => "rgba(0,0,0,0.5)",
        ],
        div![
            style![
                St::Padding => px(20),
                St::Margin => "auto",
                St::Width => percent(80),
                St::BackgroundColor => "white",
            ],
            span![
                style! {
                    St::Float => "right",
                    St::MarginTop => px(5),
                    St::MarginBottom => px(5),
                    St::Cursor => "pointer",
                },
                "[✕]",
                ev(Ev::Click, move |_| Msg::UserToggleHelp),
            ],
            help_message,
        ],
        ev(Ev::Click, move |_| Msg::UserToggleHelp),
    ]
}

fn view_blocks(
    view_cache: &ViewCache,
    state: &nakamoto_consensus::NakamotoNodeState,
    x: f32,
    y: f32,
) -> Vec<Node<Msg>> {
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
                        line_![attrs! {
                            At::X1 => x + (block_width + block_spacing) * (i as f32) + block_width / 2.,
                            At::X2 => x + (block_width + block_spacing) * (k as f32) + block_width,
                            At::Y1 => y + (block_height + block_spacing) * (j as f32),
                            At::Y2 => y + (block_height + block_spacing) * (j as f32) + block_height /2.,
                            At::Stroke => view_cache.color(nakamoto_consensus::to_number(block_map[i][j-1].unwrap())),
                        }]
                    );
                    break;
                } else {
                    result.push(rect![attrs! {
                        At::X => x + (block_width + block_spacing)* (i as f32),
                        At::Y => y + (block_height + block_spacing)* (j as f32),
                        At::Width => block_width,
                        At::Height => block_height,
                        At::Fill => view_cache.color(nakamoto_consensus::to_number(block_hash))
                    }]);
                    result.push(line_![attrs! {
                        At::X1 => x + (block_width + block_spacing) * (i as f32) + block_width / 2.,
                        At::X2 => x + (block_width + block_spacing) * (i as f32) + block_width / 2.,
                        At::Y1 => y + (block_height + block_spacing) * (j as f32) + block_height,
                        At::Y2 => y + (block_height + block_spacing) * ((j + 1) as f32),
                        At::Stroke => view_cache.color(nakamoto_consensus::to_number(block_hash)),
                    }]);
                }
            }
        }
    }
    result
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
    let progress = time_span.progress(time_now) as f32;
    // clippy said that `mul_add` could be faster...
    let x = (trajectory.end.x - trajectory.start.x).mul_add(progress, trajectory.start.x);
    let y = (trajectory.end.y - trajectory.start.y).mul_add(progress, trajectory.start.y);
    (x, y)
}
