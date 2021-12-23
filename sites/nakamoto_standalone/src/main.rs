use yew::prelude::*; // TODO make this a reexport of isds maybe? check how yew example do this

use std::cell::RefCell;
use std::rc::Rc;

struct NakamotoStandalone;

impl Component for NakamotoStandalone {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let mut sim = isds::Simulation::new();

        sim.add_event_handler(isds::InvokeProtocolForAllNodes(
            // simple_flooding::SimpleFlooding::<u32>::default(),
            // random_walks::RandomWalks::new(1024),
            isds::nakamoto_consensus::NakamotoConsensus::default(),
        ));
        sim.do_now(isds::StartAutomaticRandomNodePokes(2.));

        sim.do_now(isds::SpawnRandomNodes(32));
        sim.do_now(isds::MakeDelaunayNetwork);
        // sim.do_now(PokeMultipleRandomNodes(1));

        let sim = Rc::new(RefCell::new(sim));

        html! {
            <isds::Isds sim={ sim }>
                { "FPS: " } <isds::FpsCounter />
                <br />
                <isds::NetView />
            </isds::Isds>
        }
    }
}

fn main() {
    yew::start_app::<NakamotoStandalone>();
}
