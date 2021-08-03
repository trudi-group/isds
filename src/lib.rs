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
    pub protocol: protocols::random_walks::RandomWalks,
}

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.after_next_render(Msg::Rendered);
    let mut sim = Simulation::new();
    sim.do_now(SpawnRandomNodes(64));
    sim.do_now(MakeDelaunayNetwork);
    Model {
        sim,
        view_cache: ViewCache::new(),
        fps: FPSCounter::default(),
        protocol: random_walks::RandomWalks::new(1024),
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
    Poke(Entity),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Rendered(render_info) => {
            let elapsed_browser_seconds = render_info.timestamp_delta.unwrap_or_default() / 1000.;
            model.fps.register_render_interval(elapsed_browser_seconds);

            model.sim.catch_up(
                &mut [&mut model.view_cache],
                &mut [&mut model.protocol],
                elapsed_browser_seconds,
            );

            // FIXME
            model
                .view_cache
                .update_messages(&mut model.sim.world, model.sim.time.now());

            orders.after_next_render(Msg::Rendered);
        }
        Msg::UserPausePlay => {
            model.sim.time.toggle_paused();
        }
        Msg::Poke(node) => {
            model.sim.do_now(Poke(node));
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
