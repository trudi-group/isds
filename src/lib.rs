#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};

mod protocols;
mod sim;
mod view;
mod view_helpers;
use protocols::*;
use sim::*;
use view::view;
use view_helpers::{FPSCounter, ViewCache};

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
pub struct Model {
    pub sim: Simulation,
    pub view_cache: ViewCache,
    pub fps: FPSCounter,
    pub node_logic: Box<dyn EventHandlerMut>,
    pub poker: Box<dyn EventHandlerMut>,
}

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.after_next_render(Msg::Rendered);
    let mut sim = Simulation::new();
    let mut node_logic = Box::new(InvokeProtocolForAllNodes(
        // simple_flooding::SimpleFlooding::<u32>::default(),
        // random_walks::RandomWalks::new(1024),
        nakamoto_consensus::NakamotoConsensus::default(),
    ));
    let mut poker = Box::new(ContinuousAutomaticNodePoker::new(&mut sim, 2.));
    let mut view_cache = ViewCache::default();

    sim.do_now(SpawnRandomNodes(32));
    sim.do_now(MakeDelaunayNetwork);
    // sim.do_now(PokeMultipleRandomNodes(1));

    // TODO clean this up once init logic is nicer
    // TODO improve event handler "registration" ergonomics...
    sim.catch_up(
        &mut [&mut *node_logic, &mut *poker],
        &mut [&mut view_cache],
        100.,
    );

    Model {
        sim,
        view_cache,
        fps: FPSCounter::default(),
        node_logic,
        poker,
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Debug, Copy, Clone)]
// `Msg` describes the different events you can modify state with.
pub enum Msg {
    Rendered(RenderInfo),
    UserPausePlay,
    NodeClick(Entity),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Rendered(render_info) => {
            let elapsed_browser_seconds = render_info.timestamp_delta.unwrap_or_default() / 1000.;
            model.fps.register_render_interval(elapsed_browser_seconds);

            model.sim.catch_up(
                &mut [&mut *model.node_logic, &mut *model.poker],
                &mut [&mut model.view_cache],
                elapsed_browser_seconds,
            );
            orders.after_next_render(Msg::Rendered);
        }
        Msg::UserPausePlay => {
            model.sim.time.toggle_paused();
        }
        Msg::NodeClick(node) => {
            log!(format!("Click on {}", model.sim.name(node)));
            model.sim.do_now(PokeNode(node));
            // model
            //     .sim
            //     .do_now(protocols::simple_flooding::StartSimpleFlooding(
            //         node,
            //         rand::random::<u32>(),
            //     ));
        }
    }
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}

// ------ ------
//     Tests
// ------ ------

#[cfg(test)]
mod tests {
    // use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn tests_work() {
        assert!(true);
    }
}
