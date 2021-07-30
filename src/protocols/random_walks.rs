use super::*;

pub struct Handler;
impl EventHandlerMut for Handler {
    fn handle_event(
        &mut self,
        sim: &mut Simulation,
        event: SimEvent,
    ) -> Result<(), Box<dyn Error>> {
        use SimCommand::*;
        use SimEvent::*;
        match event {
            MessageArrived(message_ent) => {
                let (&underlay_message, &payload) = sim
                    .world
                    .query_one_mut::<(&UnderlayMessage, &RandomWalkMessage)>(message_ent)
                    .unwrap();
                sim.log(format!(
                    "{}: Got message from {}",
                    sim.name(underlay_message.dest),
                    sim.name(underlay_message.source),
                ));
                let new_source = underlay_message.dest;
                let ttl = payload.ttl;
                sim.world.despawn(message_ent).unwrap();
                if ttl > 0 {
                    self.random_step(sim, new_source, ttl).unwrap();
                } else {
                    sim.log(format!("{}: A random walk ended!", sim.name(new_source),));
                }
            }
            ExternalCommand(command) => match command {
                SpawnRandomNodes(count) => {
                    for _ in 0..count {
                        sim.spawn_random_node();
                    }
                }
                SpawnRandomMessages(count) => {
                    for _ in 0..count {
                        let node = sim.pick_random_node().unwrap();
                        generic::send_message_to_random_peer(
                            sim,
                            node,
                            RandomWalkMessage::new(100),
                        )
                        .unwrap();
                    }
                }
                StartRandomWalk(start_node, ttl) => {
                    self.random_step(sim, start_node, ttl).unwrap();
                }
                AddRandomPeersToEachNode(new_peers_min, new_peers_max) => {
                    let nodes = sim.all_nodes();
                    for node in nodes.into_iter() {
                        generic::add_random_nodes_as_peers(sim, node, new_peers_min, new_peers_max);
                    }
                }
                MakeDelaunayNetwork => {
                    generic::make_delaunay_network(sim);
                }
            },
        }
        Ok(())
    }
}
impl Handler {
    pub fn random_step(
        &self,
        sim: &mut Simulation,
        source: Entity,
        current_ttl: usize,
    ) -> Result<Entity, &str> {
        if let Some(dest) = generic::pick_random_peer(sim, source) {
            Ok(sim.send_message(source, dest, RandomWalkMessage::new(current_ttl - 1)))
        } else {
            Err("Couldn't find a suitable message destination. Not enough peers?")
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RandomWalkMessage {
    pub ttl: usize,
}
impl RandomWalkMessage {
    pub fn new(ttl: usize) -> Self {
        Self { ttl }
    }
}
