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
use protocols::*;

mod simulation;
use simulation::*;

// Describes the state.
pub struct Isds { // TODO call me "isds::main" maybe?
    pub sim: Rc<RefCell<Simulation>>,
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

#[derive(Debug, Clone)]
pub enum Msg {
    Rendered(RealSeconds),
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
            last_render: 0.,
            _render_loop_handle: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{
            < ContextProvider<ContextData> context={ ContextData { sim: self.sim.clone(), last_render: self.last_render }}>
                { for ctx.props().children.iter() }
            </ ContextProvider<ContextData>>
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(time) => {
                let elapsed_browser_seconds = time - self.last_render;
                self.sim.borrow_mut().catch_up(elapsed_browser_seconds);
                self.last_render = time;
                true
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {

        // if first_render { // TODO ? or rather something for a UI component ?
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
