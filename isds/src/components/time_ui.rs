use super::*;

// #[derive(Properties, PartialEq)]
// pub struct Props {
//     #[prop_or_default]
//     sim: Option<SharedSimulation>,
// }

#[function_component(TimeUi)]
pub fn time_ui() -> Html {
    html! {
        <div>
            <TimeControls/>
            { " | " }
            <TimeDisplay/>
        </div>
    }
}

#[function_component(TimeControls)]
pub fn time_controls() -> Html {
    let context = get_isds_context!();

    // borrowing sim here is fine because we're single-threaded

    let on_pause_play = {
        let sim = context.sim.clone();
        Callback::from(move |_| sim.borrow_mut().time.toggle_paused())
    };
    let on_slower = {
        let sim = context.sim.clone();
        Callback::from(move |_| sim.borrow_mut().time.slow_down_tenfold_clamped())
    };
    let on_faster = {
        let sim = context.sim.clone();
        Callback::from(move |_| sim.borrow_mut().time.speed_up_tenfold_clamped())
    };

    html! {
        <span>
            <button onclick={ on_pause_play }>
                if context.sim.borrow().time.paused() {
                    <span class="icon">
                        <i class="fas fa-play"></i>
                    </span>
                } else {
                    <span class="icon">
                        <i class="fas fa-pause"></i>
                    </span>
                }
            </button>
            <button onclick={ on_slower }>
                <span class="icon">
                    <i class="fas fa-backward"></i>
                </span>
            </button>
            <button onclick={ on_faster }>
                <span class="icon">
                    <i class="fas fa-forward"></i>
                </span>
            </button>
        </span>
    }
}

#[function_component(TimeDisplay)]
pub fn time_display() -> Html {
    let context = get_isds_context!();
    let sim = context.sim.borrow();

    html! {
        { format!("Sim time (s): {:.3} ({}✕)", sim.time.now(), sim.time.speed()) }
    }
}
