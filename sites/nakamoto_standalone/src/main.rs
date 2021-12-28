use yew::prelude::*; // TODO make this a reexport of isds maybe? check how yew example do this

struct NakamotoStandalone;

impl Component for NakamotoStandalone {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let sim = init_simulation();

        html! {
            <isds::Isds sim={ sim.into_shared() }>
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

fn init_simulation() -> isds::Simulation {
    let mut sim = isds::Simulation::new();
    sim.add_event_handler(isds::InvokeProtocolForAllNodes(
        // simple_flooding::SimpleFlooding::<u32>::default(),
        // random_walks::RandomWalks::new(1024),
        isds::nakamoto_consensus::NakamotoConsensus::default(),
    ));
    sim.do_now(isds::StartAutomaticRandomNodePokes(2.));
    sim.do_now(isds::SpawnRandomNodes(32));
    sim.do_now(isds::MakeDelaunayNetwork);
    sim
}

fn main() {
    let document = isds::gloo::utils::document();
    let element = document.query_selector("#app").unwrap().unwrap();
    yew::start_app_in_element::<NakamotoStandalone>(element);
}
