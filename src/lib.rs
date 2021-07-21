#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};

use hecs::{Entity, World};

mod sim;
mod time;
mod view;
mod view_helpers;
use sim::*;
use time::{SimSeconds, Time};
use view::view;
use view_helpers::{name, update_view_data, ViewCache};

static NET_MAX_X: f32 = 1000.;
static NET_MAX_Y: f32 = 1000.;
// This influences message latencies. 100ms for hosts that are very far from each other should be ~realistic.
static FLIGHT_PER_SECOND: f64 = (NET_MAX_X * 10.) as f64;

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
pub struct Model {
    pub simulator: Simulator,
    pub world: World,
    pub view_cache: ViewCache,
    pub time: Time,
}

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.after_next_render(Msg::Rendered);
    let mut simulator = Simulator::new();
    let world = World::default();
    let view_cache = ViewCache::new();
    let time = Time::new(0.02);
    simulator.schedule(
        time.sim_time(),
        SimEvent::ExternalCommand(SimCommand::SpawnRandomNodes(100)),
    );
    simulator.schedule(
        time.sim_time(),
        SimEvent::ExternalCommand(SimCommand::SpawnRandomMessages(20)),
    );
    simulator.schedule(
        time.sim_time(),
        SimEvent::ExternalCommand(SimCommand::FormConnections(200)),
    );
    Model {
        world,
        simulator,
        time,
        view_cache,
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Copy, Clone)]
// `Msg` describes the different events you can modify state with.
pub enum Msg {
    Rendered(RenderInfo),
    UserPausePlay,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Rendered(render_info) => {
            // make sure animations are updated
            let browser_seconds_past = render_info.timestamp_delta.unwrap_or_default() / 1000.;
            if model
                .time
                .advance_sim_time_by(browser_seconds_past)
                .is_some()
            {
                model
                    .simulator
                    .work_until(&mut model.world, model.time.sim_time());
                update_view_data(
                    &mut model.world,
                    &mut model.view_cache,
                    model.time.sim_time(),
                );
            }
            orders.after_next_render(Msg::Rendered);
        }
        Msg::UserPausePlay => {
            model.time.toggle_paused();
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
