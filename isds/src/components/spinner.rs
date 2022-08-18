use super::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or("In progress...".to_string())]
    pub title: String,
    #[prop_or(1.0)]
    pub spins_per_second: f64,
}

#[function_component(Spinner)]
pub fn spinner(props: &Props) -> Html {
    let context = get_isds_context!();
    let spin_progress = (context.sim.borrow().time.now() * props.spins_per_second).fract();
    let spin_progress_sampled = (spin_progress * 8.).round() / 8.; // for nicer animation

    html! {
        <span
            class="icon"
            style={ format!("transform: rotate({spin_progress_sampled}turn);") }
            title={ props.title.clone() }
        >
            <i class="fas fa-spinner" ></i>
        </span>
    }
}
