use super::*;
use dyn_clone::DynClone;
use rand_distr::{Distribution, Normal};
use std::cmp;

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

#[derive(Debug, Clone)]
pub struct MultipleTimes {
    pub command: Box<dyn Command>,
    pub times: usize,
}
impl MultipleTimes {
    pub fn new(command: impl Command + 'static, times: usize) -> Self {
        Self {
            command: Box::new(command),
            times,
        }
    }
}
impl Command for MultipleTimes {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        for _ in 0..self.times {
            self.command.execute(sim)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AtStaticIntervals {
    pub command: Box<dyn Command>,
    pub interval: SimSeconds,
    pub skip_one: bool,
}
impl AtStaticIntervals {
    pub fn new(command: impl Command + 'static, interval: SimSeconds) -> Self {
        let command = Box::new(command);
        let interval = cmp::max(OrderedFloat(f64::MIN_POSITIVE), interval);
        let skip_one = true; // skip first
        Self {
            command,
            interval,
            skip_one,
        }
    }
}
impl Command for AtStaticIntervals {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        sim.do_in(
            self.interval,
            Self {
                skip_one: false,
                ..self.clone()
            },
        );
        if !self.skip_one {
            self.command.execute(sim)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtRandomIntervals {
    pub command: Box<dyn Command>,
    pub interval_distribution: Normal<f64>,
    pub skip_one: bool,
}
impl AtRandomIntervals {
    pub fn new(command: impl Command + 'static, mean_interval: SimSeconds) -> Self {
        let command = Box::new(command);
        let interval_distribution = Normal::new(mean_interval.0, 1.).unwrap();
        let skip_one = true; // skip first
        Self {
            command,
            interval_distribution,
            skip_one,
        }
    }
    fn random_interval(&self, rng: &mut impl Rng) -> SimSeconds {
        let time = OrderedFloat(self.interval_distribution.sample(rng));
        cmp::max(OrderedFloat(f64::MIN_POSITIVE), time)
    }
}
impl Command for AtRandomIntervals {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        let interval = self.random_interval(&mut sim.rng);
        sim.do_in(
            interval,
            Self {
                skip_one: false,
                ..self.clone()
            },
        );
        if !self.skip_one {
            self.command.execute(sim)
        } else {
            Ok(())
        }
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

    #[wasm_bindgen_test]
    fn periodic_commands_work() {
        let mut sim = Simulation::new();
        sim.do_now(AtStaticIntervals::new(TestCommand, SimSeconds::from(200.)));
        sim.time.set_speed(1.);
        sim.catch_up(1001.);

        let expected = vec![true, true, true, true, true];
        let actual: Vec<bool> = sim
            .world
            .query_mut::<&bool>()
            .into_iter()
            .map(|(_, &b)| b)
            .collect();
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn randomly_periodic_commands_skip_first() {
        let mut sim = Simulation::new();
        sim.do_now(AtRandomIntervals::new(TestCommand, SimSeconds::from(2000.)));
        sim.time.set_speed(1.);
        sim.catch_up(1000.);

        let expected: Vec<bool> = vec![];
        let actual: Vec<bool> = sim
            .world
            .query_mut::<&bool>()
            .into_iter()
            .map(|(_, &b)| b)
            .collect();
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn randomly_periodic_commands_work() {
        let mut sim = Simulation::new();
        sim.do_now(AtRandomIntervals::new(TestCommand, SimSeconds::from(200.)));
        sim.time.set_speed(1.);
        sim.catch_up(1000.);

        let expected_min = 2;
        let actual = sim
            .world
            .query_mut::<&bool>()
            .into_iter()
            .map(|(_, &b)| b)
            .count();
        assert!(actual >= expected_min);
    }
}
