use super::*;
use view_helpers::EdgeType;

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
            // TODO: defs! via https://developer.mozilla.org/en-US/docs/Web/SVG/Element/marker
            model.view_cache.edges.values().map(|(edge_type, line)| {
                line_![if *edge_type == EdgeType::Undirected {
                    attrs! {
                        At::X1 => line.start.x,
                        At::Y1 => line.start.y,
                        At::X2 => line.end.x,
                        At::Y2 => line.end.y,
                        At::Stroke => "gray",
                    }
                } else {
                    attrs! {
                        At::X1 => line.start.x,
                        At::Y1 => line.start.y,
                        At::X2 => line.end.x,
                        At::Y2 => line.end.y,
                        At::Stroke => "lightgray",
                        At::StrokeDashArray => "8,8",
                    }
                }]
            }),
            model.world.query::<(&UnderlayMessage, &UnderlayPosition)>().into_iter().map(|(_, (_, pos))| {
                circle![attrs! {
                    At::Cx => pos.x,
                    At::Cy => pos.y,
                    At::R => 2.0,
                    At::Fill => "red",
                }]
            }),
            model.world.query::<(&UnderlayNodeName, &UnderlayPosition)>().into_iter().map(|(_, (_, pos))| {
                circle![attrs! {
                    At::Cx => pos.x,
                    At::Cy => pos.y,
                    At::R => 5.0,
                }]
            }),
        ],
        pre![
            model.simulator.message_log.iter().map(|(time, message)| {
                format!("{:.3}: {}\n", time, message)
            })
        ]
    ]
}
