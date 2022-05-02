use super::*;

pub struct SlowDownOnMessages {
    slow_speed: f64,
    regular_speed: f64,
    is_relevant_message: fn(Entity, &World) -> bool,
    messages_in_flight: usize,
    is_active: bool,
}
impl SlowDownOnMessages {
    pub fn new(slow_speed: f64, is_relevant_message: fn(Entity, &World) -> bool) -> Self {
        let messages_in_flight = 0;
        let regular_speed = Default::default(); // will be initialized once we detect a message
        Self {
            slow_speed,
            is_relevant_message,
            messages_in_flight,
            regular_speed,
            is_active: true,
        }
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn toggle_active(&mut self, sim: &mut Simulation) {
        if self.is_active() {
            self.deactivate(sim)
        } else {
            self.activate()
        }
    }
    pub fn activate(&mut self) {
        // we deliberately skip the complexity of counting in-flight messages in `World`
        self.is_active = true;
    }
    pub fn deactivate(&mut self, sim: &mut Simulation) {
        self.is_active = false;
        if self.messages_in_flight > 0 {
            self.messages_in_flight = 0;
            sim.time.set_speed(self.regular_speed);
        }
    }
}
impl EventHandler for SlowDownOnMessages {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>> {
        if self.is_active {
            if let Event::Node(_, event) = event {
                match event {
                    NodeEvent::MessageSent(message) => {
                        if (self.is_relevant_message)(message, &sim.world) {
                            if self.messages_in_flight == 0 {
                                self.regular_speed = sim.time.speed();
                                sim.time.set_speed(self.slow_speed);
                            }
                            self.messages_in_flight = self.messages_in_flight.saturating_add(1);
                        }
                    }
                    NodeEvent::MessageArrived(message) => {
                        if (self.is_relevant_message)(message, &sim.world) {
                            self.messages_in_flight = self.messages_in_flight.saturating_sub(1);
                            if self.messages_in_flight == 0 {
                                sim.time.set_speed(self.regular_speed);
                            }
                        }
                    }
                    _ => {}
                };
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn slow_down_and_recover_on_multiple_messages() {
        let slow_speed = 0.000023;

        let mut sim = Simulation::new();
        sim.add_event_handler(SlowDownOnMessages::new(slow_speed, |_, _| true));

        let regular_speed = sim.time.speed();

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();

        sim.send_message(node1, node2, ());
        sim.work_until(SimSeconds::from(0.0000001)); // not enough for message to arrive

        assert_eq!(slow_speed, sim.time.speed(), "didn't slow down on message");

        sim.send_message(node1, node2, ());
        sim.work_until(SimSeconds::from(0.0000001)); // not enough for message to arrive

        assert_eq!(
            slow_speed,
            sim.time.speed(),
            "something got messed up on second message"
        );

        sim.process_next_event(); // this should be the first arrive
        assert_eq!(
            slow_speed,
            sim.time.speed(),
            "reverted to regular speed while messages still in flight"
        );

        sim.process_next_event(); // this should be the seccond arrive
        assert_eq!(
            regular_speed,
            sim.time.speed(),
            "didn't recover to regular speed after second message arrived"
        );
    }

    #[wasm_bindgen_test]
    fn slow_down_from_high_speed() {
        let slow_speed = 0.; // pause so we can catch that

        let mut sim = Simulation::new();
        sim.add_event_handler(SlowDownOnMessages::new(slow_speed, |_, _| true));

        sim.time.set_speed(10000000000.);

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        sim.send_message(node1, node2, ());

        sim.catch_up(100.);
        assert_eq!(0., sim.time.speed());
    }
}
