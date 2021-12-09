use super::*;
use rand_distr::{Distribution, Normal};
use std::cmp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PokeNode(pub Entity);
impl Command for PokeNode {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        sim.schedule_now(Event::Node(self.0, NodeEvent::Poke));
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PokeRandomNode;
impl Command for PokeRandomNode {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        let node = sim
            .pick_random_node()
            .ok_or_else(|| "Not enough nodes?".to_string())?;
        sim.schedule_now(Event::Node(node, NodeEvent::Poke));
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PokeMultipleRandomNodes(pub usize);
impl Command for PokeMultipleRandomNodes {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        // TODO it is confusing that some nodes are poked twice
        for _ in 0..self.0 {
            let node = sim
                .pick_random_node()
                .ok_or_else(|| "Not enough nodes?".to_string())?;
            sim.schedule_now(Event::Node(node, NodeEvent::Poke));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PokeEachNode;
impl Command for PokeEachNode {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        for &node in sim.all_nodes().iter() {
            sim.schedule_now(Event::Node(node, NodeEvent::Poke));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StartAutomaticRandomNodePokes(pub f64);
impl Command for StartAutomaticRandomNodePokes {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        let poker = AutomaticRandomNodePokes::new(sim, self.0);
        sim.add_event_handler(poker);
        Ok(())
    }
}

pub struct AutomaticRandomNodePokes {
    timer_entity: Entity,
    poke_interval_distribution: Normal<f64>,
}
impl AutomaticRandomNodePokes {
    pub fn new(sim: &mut Simulation, mean_poke_interval: f64) -> Self {
        let timer_entity = sim.world.spawn((PokeTimer,));
        let poke_interval_distribution = Normal::new(mean_poke_interval, 1.).unwrap();
        let new_self = Self {
            timer_entity,
            poke_interval_distribution,
        };
        new_self.schedule_next_poke(sim);
        new_self
    }
    fn schedule_next_poke(&self, sim: &mut Simulation) {
        let next_poke_in = self.random_wait(&mut sim.rng);
        sim.log(format!("Next automatic poke in {:.3}...", next_poke_in));
        sim.schedule_in(next_poke_in, Event::Generic(self.timer_entity));
    }
    fn random_wait(&self, rng: &mut impl Rng) -> SimSeconds {
        let time = OrderedFloat(self.poke_interval_distribution.sample(rng));
        cmp::max(OrderedFloat(f64::MIN_POSITIVE), time)
    }
}
impl EventHandler for AutomaticRandomNodePokes {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>> {
        if let Event::Generic(entity) = event {
            if entity == self.timer_entity {
                let node = sim
                    .pick_random_node()
                    .ok_or_else(|| "Not enough nodes?".to_string())?;
                sim.schedule_now(Event::Node(node, NodeEvent::Poke));
                self.schedule_next_poke(sim);
            }
        }
        Ok(())
    }
}
struct PokeTimer;
