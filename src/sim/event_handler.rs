use super::*;

pub trait EventWatcher {
    fn handle_event(&mut self, sim: &Simulation, event: Event) -> Result<(), Box<dyn Error>>;
}

pub trait EventHandler {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>>;
}
