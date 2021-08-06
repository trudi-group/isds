use super::*;
use std::collections::BTreeMap;
use view_helpers::*;

const NCOLORS: usize = 128;

// `view` describes what to display.
#[rustfmt::skip]
pub fn view(model: &Model) -> impl IntoNodes<Msg> {
    let sim_time = model.sim.time.now();
    nodes![
        div![
            button![if model.sim.time.paused() { "Play" } else { "Pause" }, ev(Ev::Click, |_| Msg::UserPausePlay)],
            format!("Sim time (s): {:.3}", sim_time),
            format!(" | FPS: {:.0}", model.fps.get()),
        ],
        svg![
            style! {
                St::BorderStyle => "solid",
                St::Width => px(model.sim.underlay_width()),
                St::Height => px(model.sim.underlay_height()),
            },
            view_palette(),
            view_edges(model.view_cache.edges()),
            view_messages(&model.sim.world, sim_time),
            view_nodes(&model.sim.world),
        ],
        view_log(model.sim.logger.entries()),
    ]
}

fn view_palette() -> Vec<Node<Msg>> {
    (0..NCOLORS).map(|i| {
        circle![
            attrs! {
                At::Cx => 10. + (5 * i) as f32,
                At::Cy => 10.,
                At::R => 5.,
                At::Fill => color(i as u32),
            }
        ]
    }).collect()
}

fn view_nodes(world: &World) -> Vec<Node<Msg>> {
    let r = 5.0;
    world
        .query::<(
            &UnderlayPosition,
            &simple_flooding::SimpleFloodingState<u32>,
        )>()
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
                state.own_haves.iter().enumerate().map(|(i, &have)| {
                    circle![
                        attrs! {
                            At::Cx => pos.x + 1.5*r + 3.* (i as f32),
                            At::Cy => pos.y - 1.5*r,
                            At::R => 2.,
                            At::Fill => color(pseudorandomize(have))
                        }
                    ]
                })
            ]
        })
        .collect()
}

fn view_edges(edges: &BTreeMap<EdgeEndpoints, (EdgeType, UnderlayLine)>) -> Vec<Node<Msg>> {
    edges
        .values()
        .map(|(edge_type, line)| {
            line_![if *edge_type == EdgeType::Undirected {
                attrs! {
                    At::X1 => line.start.x,
                    At::Y1 => line.start.y,
                    At::X2 => line.end.x,
                    At::Y2 => line.end.y,
                    At::Stroke => "gray",
                }
            } else {
                // TODO: https://developer.mozilla.org/en-US/docs/Web/SVG/Element/marker
                attrs! {
                    At::X1 => line.start.x,
                    At::Y1 => line.start.y,
                    At::X2 => line.end.x,
                    At::Y2 => line.end.y,
                    At::Stroke => "lightgray",
                    At::StrokeDashArray => "8,8",
                }
            }]
        })
        .collect()
}

fn view_messages(world: &World, time_now: SimSeconds) -> Vec<Node<Msg>> {
    world
        .query::<(&UnderlayLine, &TimeSpan, &simple_flooding::SimpleFloodingMessage<u32>)>()
        .into_iter()
        .map(|(_, (trajectory, time_span, message))| {
            let (x, y) = message_position(trajectory, time_span, time_now);
            circle![attrs! {
                At::Cx => x,
                At::Cy => y,
                At::R => 2.0,
                At::Fill => color(pseudorandomize(message.0)),
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

fn color(number: u32) -> String {
    use palette::{Pixel, Srgb, Gradient, Lab, FromColor};

    // Weizenbaum colors...
    let wigreige = Srgb::new(166u8, 157u8, 130u8);
    let wimauve = Srgb::new(125, 80, 90);
    let wiblue = Srgb::new(35, 90, 130);
    let widarkgreen = Srgb::new(70, 105, 90);
    let wilightgreen = Srgb::new(130, 150, 100);
    let wiyellow = Srgb::new(200, 140, 40);
    let wiorange = Srgb::new(190, 85, 45);

    let wicolors = vec![
        wigreige,
        wimauve,
        wiblue,
        widarkgreen,
        wilightgreen,
        wiyellow,
        wiorange,
    ];
    // let color = wicolors[pseudorandomize(number) as usize % wicolors.len()];

    let gradient  = Gradient::new(wicolors.into_iter().map(|c| Lab::from_color(c.into_format::<f32>().into_linear())));
    let taken_colors: Vec<_> = gradient.take(NCOLORS).collect();
    let color = taken_colors[((number as usize) % NCOLORS) as usize];

    format!("#{}", hex::encode(Srgb::from_color(color).into_format().into_raw::<[u8; 3]>()))
}

fn pseudorandomize(number: u32) -> u32 {
    // idea stolen from legion's `U64Hasher`
    let big_prime: u32 = 2^31 - 1;
    big_prime.wrapping_mul(number)
}
