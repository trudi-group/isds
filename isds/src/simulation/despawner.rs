use super::*;

#[derive(Debug, Default)]
pub struct Despawner;
impl EventHandler for Despawner {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>> {
        if let Event::Node(_, node_event) = event {
            match node_event {
                NodeEvent::MessageArrived(message) => sim.world.despawn(message)?,
                NodeEvent::TimerFired(timer) => sim.world.despawn(timer)?,
                _ => (),
            }
        }
        Ok(())
    }
}
