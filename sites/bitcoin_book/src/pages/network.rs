use super::*;

#[function_component(Network)]
pub fn network() -> Html {
    html! {
        <StandardPage title="The peer-to-peer network">
            <p class="block">
                {
                    "Some day, this page might have more to tell you about the peer-to-peer
                     network that underlies Bitcoin and many other blockchain-based systems."
                }

            </p>
            <p class="block">
                {
                    "Right now, we can only offer you a bigger network to play with.
                     Try creating some forks!
                     Clicking on a node causes it to mine a block and clicking on the links between
                     nodes causes them to disappear."
                }
            </p>
            <div class="block">
                <Standalone />
            </div>
            <p class="block">
                {
                    indoc_markdown_content! { r#"
                        Pssst.... you can also use your keyboard!

                        - `[space]` ⇨ pause/play simulation
                        - `[←]`/`[→]` ⇨ control simulation speed
                        - `[m]` ⇨ a random node will "mine" a block
                        - `[t]` ⇨ a random node will send out a random transaction
                        - `[s]` ⇨ toggle slowdown on messages
                        "#
                    }
                }
            </p>
        </StandardPage>
    }
}

pub struct Standalone {
    sim: isds::SharedSimulation,
    slowdown_handler_index: usize,
    _key_listener: gloo::events::EventListener,
}

impl Component for Standalone {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let mut sim = init_simulation();

        // add handler to make time run slower when messages are in-flight
        let slowdown_handler_index =
            sim.add_event_handler(isds::SlowDownOnMessages::new(0.01, |_, _| true, true));

        // switch to high speed so we can see something happening quickly
        sim.time.set_speed(100.);

        let sim = sim.into_shared();
        let _key_listener = init_keyboard_listener(sim.clone(), slowdown_handler_index);

        Self {
            sim,
            slowdown_handler_index,
            _key_listener,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let on_node_click = {
            let sim = self.sim.clone();
            Callback::from(move |node| {
                sim.borrow_mut()
                    .do_now(isds::ForSpecific(node, isds::nakamoto_consensus::MineBlock))
            })
        };
        html! {
            <isds::Isds sim={ self.sim.clone() }>
                <div style="margin-bottom: -30px"> // chosen based on height of TimeUi level
                    <isds::TimeUi
                        slowdown_handler_index={
                            Some(self.slowdown_handler_index)
                        }
                    />
                </div>
                <isds::NetView { on_node_click } />
            </isds::Isds>
        }
    }
}

fn init_simulation() -> isds::Simulation {
    let mut sim = isds::Simulation::new();
    sim.add_event_handler(isds::InvokeProtocolForAllNodes(
        isds::nakamoto_consensus::NakamotoConsensus::default(),
    ));
    // magically mine a block at random intervals centered around 10 minutes
    sim.do_now(isds::AtRandomIntervals::new(
        isds::ForRandomNode(isds::nakamoto_consensus::MineBlock),
        isds::SimSeconds::from(600.),
    ));
    sim.do_now(isds::SpawnRandomNodes(34));
    sim.do_now(isds::DespawnMostCrowdedNodes(2));
    sim.do_now(isds::MakeDelaunayNetwork);
    sim
}
