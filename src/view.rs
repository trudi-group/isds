use super::*;
use view_helpers::*;

// `view` describes what to display.
#[rustfmt::skip]
pub fn view(model: &Model) -> impl IntoNodes<Msg> {
    let sim_time = model.time.sim_time();
    nodes![
        div![
            button![if model.time.paused { "Play" } else { "Pause" }, ev(Ev::Click, |_| Msg::UserPausePlay)],
            format!("Sim time (s): {:.3}", sim_time),
        ],
        svg![
            style! {
                St::BorderStyle => "solid",
                St::Width => px(NET_MAX_X),
                St::Height => px(NET_MAX_Y),
            },
            // model.view_cache.topology(), // based on view_topology below
            // model.view_cache.messages(), // based on view_messages below
            // view_topology(&model.world, &model.view_cache.edges()),
            view_edges(&model.view_cache.edges()),
            view_messages(&model.world),
            view_nodes(&model.world),
        ],
    ]
}

#[rustfmt::skip]
pub fn view_topology(world: &World, edges: &EdgeMap) -> Vec<Node<Msg>> {
    nodes![
        view_edges(edges),
        view_nodes(world),
    ]
}

pub fn view_nodes(world: &World) -> Vec<Node<Msg>> {
    world
        .query::<(&UnderlayNodeName, &UnderlayPosition)>()
        .into_iter()
        .map(|(_, (_, pos))| {
            circle![attrs! {
                At::Cx => pos.x,
                At::Cy => pos.y,
                At::R => 5.0,
            }]
        })
        .collect()
}

pub fn view_edges(edges: &BTreeMap<EdgeEndpoints, (EdgeType, UnderlayLine)>) -> Vec<Node<Msg>> {
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

pub fn view_messages(world: &World) -> Vec<Node<Msg>> {
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
