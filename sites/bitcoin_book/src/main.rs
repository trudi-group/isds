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
        let wallet_send_amounts = vec![0.5, 1., 5., 10.];
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
                                <isds::Wallet
                                    full_node={ Some(self.wallet_node) }
                                    address="Alice"
                                    send_whitelist={
                                        Some(isds::SendWhitelist::new(vec!["Bob", "Charlie"], wallet_send_amounts.clone()))
                                    }
                                />
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

    // init network
    sim.do_now(isds::SpawnRandomNodes(10));
    sim.do_now(isds::MakeDelaunayNetwork);
    sim.work_until(isds::SimSeconds::from(0.001)); // to make sure that some nodes are there

    // make fake "genesis payments" so that wallet balances are not 0
    let power_node = sim.pick_random_node().unwrap();
    sim.do_now(isds::ForSpecific(
        power_node,
        isds::nakamoto_consensus::BuildAndBroadcastTransaction::from(
            "CoinBroker25",
            "Alice",
            isds::blockchain_types::toshis_from(10.) as u64,
        ),
    ));
    sim.do_now(isds::ForSpecific(
        power_node,
        isds::nakamoto_consensus::BuildAndBroadcastTransaction::from(
            "Roberts",
            "Alice",
            isds::blockchain_types::toshis_from(15.) as u64,
        ),
    ));
    // bury them beneath a couple of blocks
    sim.do_now(isds::MultipleTimes::new(
        isds::ForSpecific(power_node, isds::nakamoto_consensus::MineBlock),
        3,
    ));
    sim.work_until(isds::SimSeconds::from(1.));

    // periodic logic
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(
            isds::nakamoto_consensus::BuildAndBroadcastTransaction::from(
                "Alice",
                "Bob",
                isds::blockchain_types::toshis_from(0.1337) as u64,
            ),
        ),
        isds::SimSeconds::from(5.),
    ));
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(
            isds::nakamoto_consensus::BuildAndBroadcastTransaction::from(
                "Bob",
                "Alice",
                isds::blockchain_types::toshis_from(0.42) as u64,
            ),
        ),
        isds::SimSeconds::from(5.),
    ));
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(isds::nakamoto_consensus::MineBlock),
        isds::SimSeconds::from(2.),
    ));
    sim
}

fn main() {
    yew::start_app::<BitcoinBook>();
}
