use super::*;
use user::User;

use isds::{log, SharedSimulation};

use rand::{seq::SliceRandom, thread_rng, Rng};

pub struct Layers {
    sim: SharedSimulation,
    users: Vec<User>,
    blockchain_viewing_node: Option<isds::Entity>,
    slowdown_handler_index: usize,
    _key_listener: gloo::events::EventListener,
}

impl Component for Layers {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let mut sim = init_simulation();

        // add handler to make time run slower when messages are in-flight
        let slowdown_handler_index =
            sim.add_event_handler(isds::SlowDownOnMessages::new(0.01, |_, _| true, true));

        // switch to real time
        sim.time.set_speed(1.);

        let users = vec![
            User::new("Alice", Some(sim.pick_random_node().unwrap()), true),
            User::new("Bob", Some(sim.pick_random_node().unwrap()), true),
            User::new("Charlie", None, false),
        ];

        let blockchain_viewing_node = users[0].wallet_node;

        let sim = sim.into_shared();
        let _key_listener = init_keyboard_listener(sim.clone());
        Self {
            sim,
            users,
            blockchain_viewing_node,
            slowdown_handler_index,
            _key_listener,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let wallet_send_amounts = vec![0.5, 1., 5., 10.];
        html! {
            <isds::Isds sim={ self.sim.clone() }>
                <header class="section">
                    <h1 class="title">{ "Layers of Bitcoin*" }</h1>
                    <h2 class="subtitle">{ "* and \"blockchain\" more generally" }</h2>
                    { include_markdown_content!("intro.md") }
                </header>
                <main class="section">
                    { view_layer(
                        html!{
                            <div class="columns">
                                {
                                    self.users
                                        .iter()
                                        .filter(|user| user.show_wallet)
                                        .map(|user| html!{
                                            <div class="column">
                                                <isds::Wallet
                                                    full_node={ user.wallet_node }
                                                    address={ user.name.clone() }
                                                    send_whitelist={
                                                        Some(isds::SendWhitelist::new(
                                                                self.users
                                                                    .iter()
                                                                    .filter(|u| *u != user)
                                                                    .map(|u| &u.name)
                                                                    .cloned()
                                                                    .collect(),
                                                                wallet_send_amounts.clone()
                                                            )
                                                        )
                                                    }
                                                    class="box"
                                                />
                                            </div>
                                        }).collect::<Html>()
                                }
                            </div>
                        },
                        include_markdown_content!("wallets.md")
                    )}
                    { view_layer(
                        html!{
                            <>
                                <isds::BlockchainView
                                    viewing_node={ self.blockchain_viewing_node }
                                />
                            </>
                        },
                        include_markdown_content!("blockchain.md")
                    )}
                    { view_layer(
                        html!{
                            <>
                                <isds::TimeUi
                                    show_fps=false
                                    slowdown_handler_index={
                                        Some(self.slowdown_handler_index)
                                    }
                                />
                                <isds::NetView />
                            </>
                        },
                        include_markdown_content!("network.md")
                    )}
                </main>
            </isds::Isds>
        }
    }
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

    // make some transactions so that wallet balances are not 0
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
    // mine a block
    sim.do_now(isds::ForSpecific(
        power_node,
        isds::nakamoto_consensus::MineBlock,
    ));

    // make two blocks with some arbitrary transactions (so they don't look so empty)
    let mut rng = thread_rng();
    let addresses = "CDEFGHIJKLMNOPQRSTUVWXYZ"
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();
    for _ in 0..2 {
        for _ in 0..rng.gen_range(1..5) {
            sim.do_now(isds::ForSpecific(
                power_node,
                isds::nakamoto_consensus::BuildAndBroadcastTransaction::from(
                    addresses.choose(&mut rng).unwrap(),
                    addresses.choose(&mut rng).unwrap(),
                    isds::blockchain_types::toshis_from(rng.gen_range(1..100) as f64) as u64,
                ),
            ));
        }

        // mine a block
        sim.do_now(isds::ForSpecific(
            power_node,
            isds::nakamoto_consensus::MineBlock,
        ));
    }
    sim.work_until(isds::SimSeconds::from(1.));

    // magically mine a block at random intervals centered around 10 minutes
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(isds::nakamoto_consensus::MineBlock),
        isds::SimSeconds::from(600.),
    ));

    sim
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

fn view_layer(simulation_part: Html, explanation_part: Html) -> Html {
    html! {
        <div class="columns is-reversed-desktop">
            <div class="column is-two-thirds-desktop">
                <div class="box">
                    { simulation_part }
                </div>
            </div>
            <div class="column">
                { explanation_part }
            </div>
        </div>
    }
}
