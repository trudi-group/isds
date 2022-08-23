use super::*;

#[function_component(Network)]
pub fn network() -> Html {
    html! {
        <>
            <Header title="The peer-to-peer network">
                {
                    indoc_markdown_content! { r#"
                        Some day, this page might have more to tell you about the peer-to-peer
                        networks that underlie Bitcoin and many other blockchain-based systems.

                        Right now, we can only offer you a bigger network to play with.
                        As everywhere on this website,
                        what is simulated here is an approximation of what Bitcoin is doing.
                        The main principles are the same,
                        the details vary.

                        Try creating some forks!
                        Clicking on a node causes it to mine a block.
                        Clicking on a link between two nodes will cause that link to disappear.
                        In the real world, peers can disconnect for various reasons,
                        including Internet-level problems and even deliberate attacks.
                        Can you cause a permanent network split?
                        "#
                    }
                }
            </Header>
            <Section>
                <div class="block">
                    <Standalone />
                </div>
                <div class="block">
                    <KeyboardShortcutsList />
                </div>
            </Section>
            <Footer />
        </>
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
                <isds::TimeUi
                    slowdown_handler_index={
                        Some(self.slowdown_handler_index)
                    }
                />
                <isds::NetView
                    { on_node_click }
                    buffer_space=25. // just enough for nodes and blockchains to be fully visible
                />
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
