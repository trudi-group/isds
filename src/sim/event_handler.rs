use super::*;

pub trait EventHandler {
    fn handle_event(&mut self, sim: &Simulation, event: Event) -> Result<(), Box<dyn Error>>;
}

pub trait EventHandlerMut {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>>;
}
