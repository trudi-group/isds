use isds::{log, SharedSimulation};
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[macro_use]
mod utils;

struct BitcoinBook {
    sim: SharedSimulation,
    wallet_node: isds::Entity,
    slowdown_handler_index: usize,
    _key_listener: gloo::events::EventListener,
}

impl Component for BitcoinBook {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let (sim, slowdown_handler_index) = init_simulation();
        let sim = sim.into_shared();
        let wallet_node = sim.borrow_mut().pick_random_node().unwrap();
        let _key_listener = init_keyboard_listener(sim.clone());
        Self {
            sim,
            wallet_node,
            slowdown_handler_index,
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
                    <h1 class="title">{ "Layers of Bitcoin*" }</h1>
                    <h2 class="subtitle">{ "* and \"blockchain\" more generally" }</h2>
                    { include_markdown_content!("../assets/intro.md") }
                </header>
                <main class="section">
                    <div class="columns">
                        <div class="column is-two-thirds">
                            <div class="box">
                                <isds::Isds sim={ self.sim.clone() }>
                                    <div class="columns">
                                        <div class="column">
                                            <isds::Wallet
                                                full_node={ Some(self.wallet_node) }
                                                address="Alice"
                                                send_whitelist={
                                                    Some(isds::SendWhitelist::new(
                                                            vec!["Bob", "Charlie"],
                                                            wallet_send_amounts.clone()
                                                        )
                                                    )
                                                }
                                                class="box"
                                            />
                                        </div>
                                        <div class="column">
                                            <isds::Wallet
                                                full_node={ Some(self.wallet_node) }
                                                address="Bob"
                                                send_whitelist={
                                                    Some(isds::SendWhitelist::new(
                                                            vec!["Alice", "Charlie"],
                                                            wallet_send_amounts.clone()
                                                        )
                                                    )
                                                }
                                                class="box"
                                            />
                                        </div>
                                    </div>
                                    <isds::TimeUi
                                        show_fps=true
                                        slowdown_handler_index={ Some(self.slowdown_handler_index) }
                                    />
                                    <isds::NetView />
                                </isds::Isds>
                            </div>
                        </div>
                        <div class="column">
                            { include_markdown_content!("../assets/wallets.md") }
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

fn init_simulation() -> (isds::Simulation, usize) {
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

    // magically mine a block at random intervals centered around 10 minutes
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(isds::nakamoto_consensus::MineBlock),
        isds::SimSeconds::from(600.),
    ));

    // make time run slower when messages are in-flight
    let slowdown_handler_index =
        sim.add_event_handler(isds::SlowDownOnMessages::new(0.01, |_, _| true));

    // switch to real time
    sim.time.set_speed(1.);

    (sim, slowdown_handler_index)
}

fn main() {
    yew::start_app::<BitcoinBook>();
}
