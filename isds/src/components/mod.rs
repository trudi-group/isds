use super::*;

macro_rules! get_context_data {
    ($ctx:expr, $Self:ty) => {{
        $ctx.link()
            .context::<IsdsContext>(
                $ctx.link()
                    .callback(|data: IsdsContext| <$Self>::Message::Rendered(data.last_render)),
            )
            .expect("isds context data")
    }};
}

mod fps_counter;
pub use fps_counter::FpsCounter;

mod net_view;
pub use net_view::NetView;
