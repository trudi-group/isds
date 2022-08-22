use super::*;

#[function_component(KeyboardShortcutsList)]
pub fn keyboard_shortcuts() -> Html {
    indoc_markdown_content! { r#"
        Pssst.... you can also use your keyboard!

        - `[space]` ⇨ pause/play simulation
        - `[←]`/`[→]` ⇨ control simulation speed
        - `[m]` ⇨ a random node will "mine" a block
        - `[t]` ⇨ a random node will send out a random transaction
        - `[s]` ⇨ toggle slowdown on messages
        "#
    }
}

pub fn init_keyboard_listener(
    sim: isds::SharedSimulation,
    slowdown_handler_index: usize,
) -> gloo::events::EventListener {
    init_keyboard_listener_with_block_mine_command(
        sim,
        slowdown_handler_index,
        isds::nakamoto_consensus::MineBlock,
    )
}

pub fn init_keyboard_listener_with_block_size_limit(
    sim: isds::SharedSimulation,
    slowdown_handler_index: usize,
    block_size_limit: usize,
) -> gloo::events::EventListener {
    init_keyboard_listener_with_block_mine_command(
        sim,
        slowdown_handler_index,
        isds::nakamoto_consensus::MineBlockWithLimit(block_size_limit),
    )
}

fn init_keyboard_listener_with_block_mine_command(
    sim: isds::SharedSimulation,
    slowdown_handler_index: usize,
    mine_action: impl isds::EntityAction + 'static,
) -> gloo::events::EventListener {
    let window = gloo::utils::window();
    gloo::events::EventListener::new_with_options(
        &window,
        "keydown",
        gloo::events::EventListenerOptions::enable_prevent_default(),
        move |event| {
            let e = event.clone().dyn_into::<web_sys::KeyboardEvent>().unwrap();
            match e.key().as_str() {
                " " => {
                    sim.borrow_mut().time.toggle_paused();
                    e.prevent_default()
                }
                "ArrowLeft" => {
                    sim.borrow_mut().time.slow_down_tenfold_clamped();
                    e.prevent_default()
                }
                "ArrowRight" => {
                    sim.borrow_mut().time.speed_up_tenfold_clamped();
                    e.prevent_default()
                }
                "m" => {
                    sim.borrow_mut()
                        .do_now(isds::ForRandomNode(mine_action.clone()));
                    e.prevent_default()
                }
                "t" => {
                    random_transaction_from_random_node(&mut sim.borrow_mut());
                    e.prevent_default()
                }
                "s" => {
                    let mut sim = sim.borrow_mut();
                    if let Some(slowdown_handler) =
                        sim.additional_event_handlers()
                            .borrow_mut()
                            .get_mut::<isds::SlowDownOnMessages>(slowdown_handler_index)
                    {
                        slowdown_handler.toggle_enabled(&mut sim);
                    }
                    e.prevent_default()
                }
                _ => isds::log!("Unmapped key pressed: {:?}", e),
            }
        },
    )
}
