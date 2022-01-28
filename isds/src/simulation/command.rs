use super::*;
use dyn_clone::DynClone;

pub trait Command: DynClone + std::fmt::Debug + Sync + Send {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>>;
}
dyn_clone::clone_trait_object!(Command);

pub trait EntityAction: Clone + std::fmt::Debug + Sync + Send {
    fn execute_for(&self, sim: &mut Simulation, entity: Entity) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub struct ForSpecific<A: EntityAction>(pub Entity, pub A);
impl<A: EntityAction> Command for ForSpecific<A> {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        self.1.execute_for(sim, self.0)
    }
}

impl Simulation {
    pub fn do_now(&mut self, command: impl Command + 'static) {
        self.do_at(self.time.now(), command)
    }
    pub fn do_in(&mut self, duration: SimSeconds, command: impl Command + 'static) {
        self.do_at(self.time.now() + duration, command)
    }
    pub fn do_at(&mut self, time_due: SimSeconds, command: impl Command + 'static) {
        let boxed_command: Box<dyn Command> = Box::new(command);
        let command_entry = self.world.spawn((time_due, boxed_command));
        self.schedule_at(time_due, Event::Command(command_entry))
    }
}

pub struct Handler;
impl EventHandler for Handler {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>> {
        if let Event::Command(command) = event {
            let command = sim
                .world
                .query_one_mut::<&Box<dyn Command>>(command)
                .unwrap();
            let command: Box<dyn Command> = dyn_clone::clone_box(&**command);
            command.execute(sim)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Debug, Clone, Copy)]
    struct TestCommand;
    impl Command for TestCommand {
        fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
            sim.world.spawn((true,));
            Ok(())
        }
    }

    #[wasm_bindgen_test]
    fn commands_work() {
        let mut sim = Simulation::new();
        sim.do_now(TestCommand);
        sim.catch_up(1000.);

        let expected = vec![true];
        let actual: Vec<bool> = sim
            .world
            .query_mut::<&bool>()
            .into_iter()
            .map(|(_, &b)| b)
            .collect();
        assert_eq!(expected, actual);
    }
}
