use isds::{log, SharedSimulation};
use wasm_bindgen::JsCast;
use yew::prelude::*;

struct NakamotoStandalone {
    sim: SharedSimulation,
    _key_listener: gloo::events::EventListener,
}

impl Component for NakamotoStandalone {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let sim = init_simulation().into_shared();
        let _key_listener = init_keyboard_listener(sim.clone());
        Self { sim, _key_listener }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <isds::Isds sim={ self.sim.clone() }>
                <div style="margin-bottom: -50px"> // chosen based on `buffer_space` of `NetView`
                    <div class="is-flex">
                        <isds::TimeUi />
                        <div class="mx-1 p-1">
                            { "FPS: " } <isds::FpsCounter />
                        </div>
                    </div>
                </div>
                <isds::NetView />
            </isds::Isds>
        }
    }
}

fn init_keyboard_listener(sim: SharedSimulation) -> gloo::events::EventListener {
    let window = gloo::utils::window();
    gloo::events::EventListener::new(&window, "keydown", move |event| {
        let e = event.clone().dyn_into::<web_sys::KeyboardEvent>().unwrap();
        match e.key().as_str() {
            " " => sim.borrow_mut().time.toggle_paused(),
            "ArrowLeft" => sim.borrow_mut().time.slow_down_tenfold_clamped(),
            "ArrowRight" => sim.borrow_mut().time.speed_up_tenfold_clamped(),
            "m" => sim.borrow_mut().do_now(isds::ForRandomNode(isds::PokeNode)),
            _ => log!("Unmapped key pressed: {:?}", e),
        }
    })
}

fn init_simulation() -> isds::Simulation {
    let mut sim = isds::Simulation::new();
    sim.add_event_handler(isds::InvokeProtocolForAllNodes(
        // simple_flooding::SimpleFlooding::<u32>::default(),
        // random_walks::RandomWalks::new(1024),
        isds::nakamoto_consensus::NakamotoConsensus::default(),
    ));
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(isds::PokeNode),
        isds::SimSeconds::from(2.),
    ));
    sim.do_now(isds::SpawnRandomNodes(32));
    sim.do_now(isds::MakeDelaunayNetwork);
    sim
}

fn main() {
    let document = gloo::utils::document();
    let element = document.query_selector("#app").unwrap().unwrap();
    yew::start_app_in_element::<NakamotoStandalone>(element);
}
