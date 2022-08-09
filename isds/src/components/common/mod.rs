use super::*;

#[macro_export]
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

mod colors;
pub use colors::*;

mod highlight;
pub use highlight::Highlight;
