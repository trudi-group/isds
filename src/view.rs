use super::*;
use std::collections::BTreeMap;
use view_helpers::*;

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
            view_edges(model.view_cache.edges()),
            view_messages(&model.sim.world),
            view_nodes(&model.sim.world),
        ],
        view_log(model.sim.logger.entries()),
    ]
}

fn view_nodes(world: &World) -> Vec<Node<Msg>> {
    world
        .query::<(&UnderlayNodeName, &UnderlayPosition)>()
        .into_iter()
        .map(|(node, (_, pos))| {
            circle![
                attrs! {
                    At::Cx => pos.x,
                    At::Cy => pos.y,
                    At::R => 5.0,
                },
                ev(Ev::Click, move |_| Msg::Poke(node)),
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

fn view_messages(world: &World) -> Vec<Node<Msg>> {
    // TODO evaluate how bad it would be to calculate the position here
    world
        .query::<(&UnderlayMessage, &UnderlayPosition)>()
        .into_iter()
        .map(|(_, (_, pos))| {
            circle![attrs! {
                At::Cx => pos.x,
                At::Cy => pos.y,
                At::R => 2.0,
                At::Fill => "red",
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
