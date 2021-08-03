use super::*;

#[derive(Debug, Default)]
pub struct Despawner;
impl EventHandlerMut for Despawner {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>> {
        match event {
            Event::Node(_, node_event) => {
                if let NodeEvent::MessageArrived(message) = node_event {
                    sim.world.despawn(message)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}
