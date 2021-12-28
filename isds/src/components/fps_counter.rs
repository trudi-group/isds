use super::*;

#[derive(Debug)]
pub struct FpsCounter {
    fps_sample: f64,
    last_render_at: RealSeconds,
    last_sample_at: RealSeconds,
    _context_handle: yew::context::ContextHandle<IsdsContext>,
}
impl FpsCounter {
    fn register_render(&mut self, render_at: RealSeconds) -> bool {
        let time_elapsed = render_at - self.last_sample_at;
        if time_elapsed > 0.5 {
            let last_interval = render_at - self.last_render_at;
            self.fps_sample = 1. / last_interval;
            self.last_sample_at = render_at;
            self.last_render_at = render_at;
            true
        } else {
            self.last_render_at = render_at;
            false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Rendered(RealSeconds),
}

impl Component for FpsCounter {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (_, _context_handle) = get_isds_context!(ctx, Self);
        Self {
            fps_sample: 0.,
            last_render_at: 0.,
            last_sample_at: 0.,
            _context_handle,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! { format!("{:.0}", self.fps_sample) }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(time) => self.register_render(time),
        }
    }
}
