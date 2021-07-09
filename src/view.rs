use super::*;

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
            <&UnderlayPosition>::query().filter(component::<UnderlayMessage>()).iter(&model.world).map(|pos| {
                circle![attrs! {
                    At::Cx => pos.x,
                    At::Cy => pos.y,
                    At::R => 2.0,
                    At::Fill => "red",
                }]
            }),
            <&UnderlayPosition>::query().filter(component::<UnderlayNodeId>()).iter(&model.world).map(|pos| {
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
