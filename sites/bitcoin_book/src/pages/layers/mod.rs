use super::*;
use user_model::User;

mod layer_description;
use layer_description::LayerDescription;

use rand::{thread_rng, Rng};

pub struct Layers {
    sim: isds::SharedSimulation,
    users: Vec<User>,
    blockchain_viewing_node: Option<isds::Entity>,
    slowdown_handler_index: usize,
    _key_listener: gloo::events::EventListener,
}

// Specifying the desired block size limit... (in number of transactions).
const BLOCK_SIZE_LIMIT: usize = 5;
const MINE_BLOCK: isds::nakamoto_consensus::MineBlockWithLimit =
    isds::nakamoto_consensus::MineBlockWithLimit(BLOCK_SIZE_LIMIT);

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
        let _key_listener = init_keyboard_listener(sim.clone(), slowdown_handler_index);
        Self {
            sim,
            users,
            blockchain_viewing_node,
            slowdown_handler_index,
            _key_listener,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <isds::Isds sim={ self.sim.clone() }>
                <header class="section"> // custom header as this is the "front page"
                    <div class="container">
                        <h1 class="title">{ "Layers of Bitcoin*" }</h1>
                        <h2 class="subtitle">{ "* and blockchain-based systems more generally" }</h2>
                        { include_markdown_content!("intro.md") }
                        <div class="notification is-hidden-tablet">
                            <span>
                                {
                                    "This page should also work on smartphones,
                                    but for the best experience,
                                    we recommend using a wider screen.
                                    And perhaps also a mouse..."
                                }
                            </span>
                            <span class="icon"><i class="fas fa-desktop"></i></span>
                            <span class="icon"><i class="fas fa-mouse"></i></span>
                        </div>
                    </div>
                </header>
                <Section>
                    { self.view_application_layer() }
                    { self.view_blockchain_layer() }
                    { self.view_consensus_layer() }
                    { self.view_network_layer() }
                </Section>
                <Section>
                    <h2 class="title is-4">{ "...and beyond..." }</h2>
                    { include_markdown_content!("outro.md") }
                </Section>
                <Footer />
            </isds::Isds>
        }
    }
}

impl Layers {
    fn view_application_layer(&self) -> Html {
        let wallet_send_amounts = vec![0.5, 1., 5., 10.];
        view_layer(
            html! {
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
            html! {
                <LayerDescription title="Application">
                    { include_markdown_content!("application.md") }
                </LayerDescription>
            },
        )
    }

    fn view_blockchain_layer(&self) -> Html {
        view_layer(
            html! {
                <isds::BlockchainView
                    viewing_node={ self.blockchain_viewing_node }
                    highlight_class={ "has-fill-info" }
                />
            },
            html! {
                <LayerDescription title="Blockchain">
                    { include_markdown_content!("blockchain.md") }
                </LayerDescription>
            },
        )
    }

    fn view_consensus_layer(&self) -> Html {
        let on_button = {
            let sim = self.sim.clone();
            Callback::from(move |_| sim.borrow_mut().do_now(isds::ForRandomNode(MINE_BLOCK)))
        };
        view_layer(
            html! {
                <div class="has-text-centered p-5">
                    <span class="is-size-4 m-5">
                        <isds::Spinner
                            title={ "Mining in progress..." }
                            spins_per_second={ 10. }
                        />
                    </span>
                    <button
                        class="button is-large"
                        onclick={ on_button }
                        title="Help a random node mine a block"
                    >
                        <span class="icon"><i class="fas fa-magic"></i></span>
                        <span class="icon"><i class="fas fa-dice"></i></span>
                    </button>
                </div>
            },
            html! {
                <LayerDescription title="Consensus">
                    { include_markdown_content!("consensus.md") }
                </LayerDescription>
            },
        )
    }

    fn view_network_layer(&self) -> Html {
        let on_node_click = {
            let sim = self.sim.clone();
            Callback::from(move |node| random_transaction(&mut sim.borrow_mut(), node))
        };
        view_layer(
            html! {
                <>
                    <isds::TimeUi
                        show_fps=false
                        slowdown_handler_index={
                            Some(self.slowdown_handler_index)
                        }
                    />
                    <isds::NetView
                        { on_node_click }
                        node_highlight_on_hover={ true }
                        highlight_class={ "has-fill-info" }
                        buffer_space=25.
                    />
                </>
            },
            html! {
                <LayerDescription title="Network">
                    { include_markdown_content!("network.md") }
                </LayerDescription>
            },
        )
    }
}

fn init_simulation() -> isds::Simulation {
    let mut sim = isds::Simulation::new_with_underlay_dimensions(320., 160.);
    sim.add_event_handler(isds::InvokeProtocolForAllNodes(
        isds::nakamoto_consensus::NakamotoConsensus::new_with_block_limit(BLOCK_SIZE_LIMIT),
    ));

    // init network
    sim.do_now(isds::SpawnRandomNodes(20));
    sim.do_now(isds::DespawnMostCrowdedNodes(10));
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
    sim.do_now(isds::ForSpecific(power_node, MINE_BLOCK));

    // make two blocks with some arbitrary transactions (so they don't look so empty)
    for _ in 0..2 {
        for _ in 0..thread_rng().gen_range(1..5) {
            random_transaction(&mut sim, power_node);
        }

        // mine a block
        sim.do_now(isds::ForSpecific(power_node, MINE_BLOCK));
    }
    sim.work_until(isds::SimSeconds::from(1.));

    // magically mine a block at random intervals centered around 10 minutes
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(MINE_BLOCK),
        isds::SimSeconds::from(600.),
    ));

    sim
}

fn view_layer(simulation_part: Html, explanation_part: Html) -> Html {
    html! {
        <div class="columns is-reversed-desktop is-centered pb-2">
            <div class="column is-two-thirds-desktop">
                <div class="box">
                    { simulation_part }
                </div>
            </div>
            <div class="column is-2-desktop is-wide-enough">
                { explanation_part }
            </div>
        </div>
    }
}
