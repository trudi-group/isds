use super::*;

macro_rules! get_isds_context {
    // for regular components...
    ($ctx:expr, $Self:ty) => {{
        $ctx.link()
            .context::<IsdsContext>(
                $ctx.link()
                    .callback(|data: IsdsContext| <$Self>::Message::Rendered(data.last_render)),
            )
            .expect("no isds context found")
    }};
    // for functional components...
    () => {{
        use_context::<IsdsContext>().expect("no isds context found")
    }};
}

mod fps_counter;
pub use fps_counter::FpsCounter;

mod net_view;
pub use net_view::NetView;

mod time_ui;
pub use time_ui::{TimeControls, TimeDisplay, TimeUi};

mod wallet;
pub use wallet::{SendWhitelist, Wallet};
