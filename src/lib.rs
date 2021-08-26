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
    pub show_help: bool,
    pub show_debug_infos: bool,
}

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.after_next_render(Msg::Rendered);
    orders.stream(streams::window_event(Ev::KeyDown, |event| {
        Msg::KeyDown(event.unchecked_into())
    }));

    let mut sim = Simulation::new();
    sim.add_event_handler(InvokeProtocolForAllNodes(
        // simple_flooding::SimpleFlooding::<u32>::default(),
        // random_walks::RandomWalks::new(1024),
        nakamoto_consensus::NakamotoConsensus::default(),
    ));
    sim.do_now(StartAutomaticRandomNodePokes(2.));
    let view_cache = ViewCache::default();

    sim.do_now(SpawnRandomNodes(32));
    sim.do_now(MakeDelaunayNetwork);
    // sim.do_now(PokeMultipleRandomNodes(1));

    // sim.catch_up_with_watchers(
    //     &mut [&mut view_cache],
    //     100.,
    // );

    Model {
        sim,
        view_cache,
        fps: FPSCounter::default(),
        show_help: true,
        show_debug_infos: false,
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Debug, Clone)]
// `Msg` describes the different events you can modify state with.
pub enum Msg {
    Rendered(RenderInfo),
    UserPausePlay,
    UserMakeFaster,
    UserMakeSlower,
    UserToggleHelp,
    NodeClick(Entity),
    LinkClick(Entity, Entity),
    KeyDown(web_sys::KeyboardEvent),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Rendered(render_info) => {
            let elapsed_browser_seconds = render_info.timestamp_delta.unwrap_or_default() / 1000.;
            model.fps.register_render_interval(elapsed_browser_seconds);

            model
                .sim
                .catch_up_with_watchers(&mut [&mut model.view_cache], elapsed_browser_seconds);
            orders.after_next_render(Msg::Rendered);
        }
        Msg::UserPausePlay => {
            model.sim.time.toggle_paused();
        }
        Msg::UserMakeFaster => {
            model
                .sim
                .time
                .set_speed((model.sim.time.speed() * 10f64).min(1000f64));
        }
        Msg::UserMakeSlower => {
            model.sim.time.set_speed(model.sim.time.speed() / 10f64);
        }
        Msg::UserToggleHelp => {
            model.show_help = !model.show_help;
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
        Msg::LinkClick(node1, node2) => {
            log!(format!(
                "Click on link between {} and {}.",
                model.sim.name(node1),
                model.sim.name(node2)
            ));
            if model
                .view_cache
                .edge_type(node1, node2)
                .unwrap()
                .is_phantom()
            {
                model.sim.do_now(AddPeer(node1, node2));
                model.sim.do_now(AddPeer(node2, node1));
            } else {
                model.sim.do_now(RemovePeer(node1, node2));
                model.sim.do_now(RemovePeer(node2, node1));
            }
        }
        Msg::KeyDown(keyboard_event) => match keyboard_event.key().as_str() {
            " " => {
                orders.send_msg(Msg::UserPausePlay);
            }
            "ArrowLeft" | "h" => {
                orders.send_msg(Msg::UserMakeSlower);
            }
            "ArrowRight" | "l" => {
                orders.send_msg(Msg::UserMakeFaster);
            }
            "m" => {
                model.sim.do_now(PokeRandomNode);
            }
            "?" => {
                orders.send_msg(Msg::UserToggleHelp);
            }
            "d" => {
                model.show_debug_infos = !model.show_debug_infos;
            }
            "Escape" => {
                model.show_help = false;
            }
            key => {
                log!("Unmapped key pressed: {:?}", key);
            }
        },
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
