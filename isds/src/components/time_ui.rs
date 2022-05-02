use super::*;

#[derive(Properties, PartialEq)]
pub struct TimeUiProps {
    #[prop_or_default]
    pub show_fps: bool,
    #[prop_or_default]
    pub slowdown_handler_index: Option<usize>,
}

#[function_component(TimeUi)]
pub fn time_ui(props: &TimeUiProps) -> Html {
    html! {
        <div class="level is-mobile">
            <div class="level-left">
                <div class="level-item">
                    <div class="buttons are-small">
                        <TimeControls/>
                    </div>
                </div>
                <div class="level-item">
                    <TimeDisplay/>
                </div>
                if props.show_fps {
                    <div class="level-item">
                        { "FPS: " } <FpsCounter />
                    </div>
                }
            </div>
                if props.slowdown_handler_index.is_some() {
                    <div class="level-right">
                        <div class="level-item">
                            <SlowdownCheckbox handler_index={ props.slowdown_handler_index }/>
                        </div>
                    </div>
                }
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
        <>
            <button class="button" onclick={ on_pause_play }>
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
            <button class="button" onclick={ on_slower }>
                <span class="icon">
                    <i class="fas fa-backward"></i>
                </span>
            </button>
            <button class="button" onclick={ on_faster }>
                <span class="icon">
                    <i class="fas fa-forward"></i>
                </span>
            </button>
        </>
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

#[derive(Properties, PartialEq)]
pub struct SlowdownCheckboxProps {
    pub handler_index: Option<usize>,
}
#[function_component(SlowdownCheckbox)]
pub fn slowdown_checkbox(props: &SlowdownCheckboxProps) -> Html {
    let context = get_isds_context!();

    let config_ok = props.handler_index.is_some();

    let slowdown_is_active = props
        .handler_index
        .and_then(|i| {
            context
                .sim
                .borrow()
                .additional_event_handlers()
                .borrow()
                .get::<SlowDownOnMessages>(i)
                .map(|h| h.is_active())
        })
        .unwrap_or(false);

    let toggle_slowdown = {
        if let Some(handler_index) = props.handler_index {
            Callback::from(move |_| {
                let mut sim = context.sim.borrow_mut();
                if let Some(slowdown_handler) = sim
                    .additional_event_handlers()
                    .borrow_mut()
                    .get_mut::<SlowDownOnMessages>(handler_index)
                {
                    slowdown_handler.toggle_active(&mut sim);
                }
            })
        } else {
            Callback::noop()
        }
    };

    html! {
        <>
            <div class="checkbox" disabled={ !config_ok }>
                <input type="checkbox" onclick={ toggle_slowdown } checked={ slowdown_is_active } />
                <label class="ml-1">
                    { "slow down on messages" }
                </label>
            </div>
        </>
    }
}
