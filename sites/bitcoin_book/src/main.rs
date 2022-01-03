use isds::{log, SharedSimulation};
use wasm_bindgen::JsCast;
use yew::prelude::*;

struct BitcoinBook {
    sim: SharedSimulation,
    wallet_node: isds::Entity,
    _key_listener: gloo::events::EventListener,
}

impl Component for BitcoinBook {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let sim = init_simulation().into_shared();
        let wallet_node = sim.borrow_mut().pick_random_node().unwrap();
        let _key_listener = init_keyboard_listener(sim.clone());
        Self {
            sim,
            wallet_node,
            _key_listener,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <>
                <header class="section">
                    // <nav class="breadcrumb" aria-label="navigation">
                    //     <ul>
                    //         <li class="is-active">{"Start"}</li>
                    //     </ul>
                    // </nav>
                    <article>
                        <h1 class="title">{ "How does Bitcoin work?" }</h1>
                        <p>
                            { "Optional text" }
                        </p>
                    </article>
                </header>
                <main class="section">
                    <div class="columns">
                        <div class="box column">
                            <isds::Isds sim={ self.sim.clone() }>
                                <isds::Wallet full_node={ Some(self.wallet_node) }/>
                                    <div class="is-flex">
                                        <isds::TimeUi />
                                        <div class="mx-1 p-1">
                                            { "FPS: " } <isds::FpsCounter />
                                        </div>
                                    </div>
                                <isds::NetView />
                            </isds::Isds>
                        </div>
                        <div class="column">
                            {"Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."}
                        </div>
                    </div>
                </main>
            </>
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
    let mut sim = isds::Simulation::new_with_underlay_dimensions(400., 200.);
    sim.add_event_handler(isds::InvokeProtocolForAllNodes(
        isds::nakamoto_consensus::NakamotoConsensus::default(),
    ));
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(isds::nakamoto_consensus::BuildAndBroadcastTransaction::new(
            "Alice", "Bob", 1337,
        )),
        isds::SimSeconds::from(0.5),
    ));
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(isds::nakamoto_consensus::MineBlock),
        isds::SimSeconds::from(2.),
    ));
    sim.do_now(isds::SpawnRandomNodes(10));
    sim.do_now(isds::MakeDelaunayNetwork);
    sim.catch_up(0.0001); // to make sure that some nodes are there
    sim
}

fn main() {
    yew::start_app::<BitcoinBook>();
}
