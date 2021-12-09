#![allow(clippy::wildcard_imports)]

#![macro_use]
extern crate gloo;
use gloo::console::log;
use gloo::render::{request_animation_frame, AnimationFrame};

use yew::prelude::*;

mod protocols;
mod sim;
mod view;
mod view_helpers;
use protocols::*;
use sim::*;
use view::*;
use view_helpers::{FPSCounter, ViewCache};

// Describes the state.
pub struct Isds {
    pub sim: Simulation,
    pub view_cache: ViewCache,
    pub fps: FPSCounter,
    pub show_help: bool,
    pub show_debug_infos: bool,
    last_render: RealSeconds,
    _render_loop_handle: Option<AnimationFrame>,
}

// Describes the different events you can modify state with.
#[derive(Debug, Clone)]
pub enum Msg {
    Rendered(RealSeconds),
    UserPausePlay,
    UserMakeFaster,
    UserMakeSlower,
    UserToggleHelp,
    NodeClick(Entity),
    LinkClick(Entity, Entity),
    KeyDown(KeyboardEvent),
}

impl Component for Isds {
    type Message = Msg;
    type Properties = (); // TODO

    fn create(ctx: &Context<Self>) -> Self {
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

        Self {
            sim,
            view_cache,
            fps: FPSCounter::default(),
            show_help: true,
            show_debug_infos: true,
            last_render: 0.,
            _render_loop_handle: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let buffer_space = 50.;
        html!{
            <>
                { self.view_menu_bar(ctx) }
                <svg
                   viewBox={ format!("{} {} {} {}",
                       -buffer_space,
                       -buffer_space,
                       self.sim.underlay_width() + 2. * buffer_space,
                       self.sim.underlay_height() + 2. * buffer_space
                    ) }
                    style=r#"border-style: "solid"; max-width: 1024px"#
                >
                    if self.show_debug_infos {
                        { view_palette(&self.view_cache) }
                    }
                    { view_edges(&self.view_cache, ctx) }
                    { view_nodes(&self.sim.world, &self.view_cache, ctx) }
                    { view_messages(&self.sim.world, &self.view_cache, self.sim.time.now()) }
                </svg>
                if self.show_debug_infos {
                    { view_log(self.sim.logger.entries()) }
                }
                // { view_help(self.show_help) }
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(render_timestamp) => {
                let elapsed_browser_seconds = render_timestamp - self.last_render;
                self.last_render = render_timestamp;

                self.fps.register_render_interval(elapsed_browser_seconds);
                self
                    .sim
                    .catch_up_with_watchers(&mut [&mut self.view_cache], elapsed_browser_seconds);
            }
            Msg::UserPausePlay => {
                self.sim.time.toggle_paused();
            }
            Msg::UserMakeFaster => {
                self
                    .sim
                    .time
                    .set_speed((self.sim.time.speed() * 10f64).min(1000f64));
            }
            Msg::UserMakeSlower => {
                self.sim.time.set_speed(self.sim.time.speed() / 10f64);
            }
            Msg::UserToggleHelp => {
                self.show_help = !self.show_help;
            }
            Msg::NodeClick(node) => {
                log!(format!("Click on {}", self.sim.name(node)));
                self.sim.do_now(PokeNode(node));
                // self
                //     .sim
                //     .do_now(protocols::simple_flooding::StartSimpleFlooding(
                //         node,
                //         rand::random::<u32>(),
                //     ));
            }
            Msg::LinkClick(node1, node2) => {
                log!(format!(
                    "Click on link between {} and {}.",
                    self.sim.name(node1),
                    self.sim.name(node2)
                ));
                if self
                    .view_cache
                    .edge_type(node1, node2)
                    .unwrap()
                    .is_phantom()
                {
                    self.sim.do_now(AddPeer(node1, node2));
                    self.sim.do_now(AddPeer(node2, node1));
                } else {
                    self.sim.do_now(RemovePeer(node1, node2));
                    self.sim.do_now(RemovePeer(node2, node1));
                }
            }
            Msg::KeyDown(keyboard_event) => match keyboard_event.key().as_str() {
                " " => {
                    ctx.link().send_message(Msg::UserPausePlay);
                }
                "ArrowLeft" | "h" => {
                    ctx.link().send_message(Msg::UserMakeSlower);
                }
                "ArrowRight" | "l" => {
                    ctx.link().send_message(Msg::UserMakeFaster);
                }
                "m" => {
                    self.sim.do_now(PokeRandomNode);
                }
                "?" => {
                    ctx.link().send_message(Msg::UserToggleHelp);
                }
                "d" => {
                    self.show_debug_infos = !self.show_debug_infos;
                }
                "Escape" => {
                    self.show_help = false;
                }
                key => {
                    log!("Unmapped key pressed: {:?}", key);
                }
            },
        }
        true // FIXME
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {

        // if first_render {
        //     let window = gloo::utils::window();
        //     window.add_event_listener_with_callback(
        //         "onkeydown",
        //         ctx.link().callback(move |e: KeyboardEvent| { e.prevent_default(); Msg::KeyDown(e) })
        //     );
        // }

        // code inspired by yew's webgl example
        let handle = {
            let link = ctx.link().clone();
            request_animation_frame(move |time| link.send_message(Msg::Rendered(time / 1000.)))
        };
        // A reference to the handle must be stored, otherwise it is dropped and the render won't
        // occur.
        self._render_loop_handle = Some(handle);
    }

    fn destroy(&mut self, ctx: &Context<Self>) {}
}

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
