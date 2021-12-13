#![allow(clippy::wildcard_imports)]

#![macro_use]
extern crate gloo;
use gloo::console::log;
use gloo::render::{request_animation_frame, AnimationFrame};

pub use yew;
use yew::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;

mod components;
pub use components::*;

mod protocols;
mod sim;
use protocols::*;
use sim::*;

// Describes the state.
pub struct Isds { // TODO call me "isds::main" maybe?
    pub sim: Rc<RefCell<Simulation>>,
    pub show_help: bool,
    pub show_debug_infos: bool,
    last_render: RealSeconds,
    _render_loop_handle: Option<AnimationFrame>,
}

#[derive(Clone)]
pub struct ContextData {
    pub sim: Rc<RefCell<Simulation>>,
    pub last_render: RealSeconds,
}
impl PartialEq for ContextData {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.sim, &other.sim) && self.last_render == other.last_render
    }
}
impl std::fmt::Debug for ContextData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").field("sim", &"hidden").field("last_render", &self.last_render).finish()
    }
}

// Describes the different events you can modify state with.
#[derive(Debug, Clone)]
pub enum Msg {
    Rendered(RealSeconds),
    // UserPausePlay,
    // UserMakeFaster,
    // UserMakeSlower,
    // UserToggleHelp,
    // NodeClick(Entity),
    // LinkClick(Entity, Entity),
    // KeyDown(KeyboardEvent),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

impl Component for Isds {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(
            // simple_flooding::SimpleFlooding::<u32>::default(),
            // random_walks::RandomWalks::new(1024),
            nakamoto_consensus::NakamotoConsensus::default(),
        ));
        sim.do_now(StartAutomaticRandomNodePokes(2.));

        sim.do_now(SpawnRandomNodes(32));
        sim.do_now(MakeDelaunayNetwork);
        // sim.do_now(PokeMultipleRandomNodes(1));

        sim.catch_up(
        //     &mut [&mut view_cache],
            100.,
        );

        Self {
            sim: Rc::new(RefCell::new(sim)),
            show_help: true,
            show_debug_infos: true,
            last_render: 0.,
            _render_loop_handle: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{
            <>
                // { self.view_menu_bar(ctx) }
                < ContextProvider<ContextData> context={ ContextData { sim: self.sim.clone(), last_render: self.last_render }}>
                    { for ctx.props().children.iter() }
                </ ContextProvider<ContextData>>
                // if self.show_debug_infos {
                //     { view_log(self.sim.logger.entries()) }
                // }
                // { view_help(self.show_help) }
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(time) => {
                let elapsed_browser_seconds = time - self.last_render;
                self.sim.borrow_mut().catch_up(elapsed_browser_seconds);
                self.last_render = time;
                true
            }
            // Msg::UserPausePlay => {
            //     self.sim.borrow_mut().time.toggle_paused();
            // }
            // Msg::UserMakeFaster => {
            //     let new_speed = (self.sim.borrow().time.speed() * 10f64).min(1000f64);
            //     self.sim.borrow_mut().time.set_speed(new_speed);
            // }
            // Msg::UserMakeSlower => {
            //     let new_speed = self.sim.borrow().time.speed() / 10f64;
            //     self.sim.borrow_mut().time.set_speed(new_speed);
            // }
            // Msg::UserToggleHelp => {
            //     self.show_help = !self.show_help;
            // }
            // Msg::KeyDown(keyboard_event) => match keyboard_event.key().as_str() {
            //     " " => {
            //         ctx.link().send_message(Msg::UserPausePlay);
            //     }
            //     "ArrowLeft" | "h" => {
            //         ctx.link().send_message(Msg::UserMakeSlower);
            //     }
            //     "ArrowRight" | "l" => {
            //         ctx.link().send_message(Msg::UserMakeFaster);
            //     }
            //     "m" => {
            //         self.sim.borrow_mut().do_now(PokeRandomNode);
            //     }
            //     "?" => {
            //         ctx.link().send_message(Msg::UserToggleHelp);
            //     }
            //     "d" => {
            //         self.show_debug_infos = !self.show_debug_infos;
            //     }
            //     "Escape" => {
            //         self.show_help = false;
            //     }
            //     key => {
            //         log!("Unmapped key pressed: {:?}", key);
            //     }
            // },
        }
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
