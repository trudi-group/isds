use super::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SpawnRandomNodes(pub usize);
impl Command for SpawnRandomNodes {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        for _ in 0..self.0 {
            sim.spawn_random_node();
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DespawnMostCrowdedNodes(pub usize);
impl Command for DespawnMostCrowdedNodes {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        for _ in 0..self.0 {
            sim.despawn_most_crowded_node()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForRandomNode<A: EntityAction>(pub A);
impl<A: EntityAction> Command for ForRandomNode<A> {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        let node = sim
            .pick_random_node()
            .ok_or_else(|| "Not enough nodes?".to_string())?;
        self.0.execute_for(sim, node)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForEachNode<A: EntityAction>(pub A);
impl<A: EntityAction> Command for ForEachNode<A> {
    fn execute(&self, sim: &mut Simulation) -> Result<(), Box<dyn Error>> {
        for &node in sim.all_nodes().iter() {
            self.0.execute_for(sim, node)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnderlayConfig {
    width: f32,
    height: f32,
    message_speed: f64,
}
impl UnderlayConfig {
    pub fn new(width: f32, height: f32) -> Self {
        // Latencies of 100ms for hosts that are very far from each other should be ~realistic.
        let message_speed = 10. * f32::max(width, height) as f64;
        Self {
            width,
            height,
            message_speed,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnderlayNodeName(pub String);

#[derive(Debug, Copy, Clone)]
pub struct UnderlayPosition {
    pub x: f32,
    pub y: f32,
}
impl UnderlayPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn distance(point1: Self, point2: Self) -> f32 {
        let x = (point1.x - point2.x).abs();
        let y = (point1.y - point2.y).abs();
        x.hypot(y)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct UnderlayLine {
    pub start: UnderlayPosition,
    pub end: UnderlayPosition,
}
impl UnderlayLine {
    pub fn from_nodes(world: &World, source: Entity, dest: Entity) -> Self {
        let start = *world.get::<UnderlayPosition>(source).unwrap();
        let end = *world.get::<UnderlayPosition>(dest).unwrap();
        Self { start, end }
    }
    pub fn length(&self) -> f32 {
        UnderlayPosition::distance(self.start, self.end)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct UnderlayMessage {
    pub source: Entity,
    pub dest: Entity,
}

impl Simulation {
    pub fn underlay_width(&self) -> f32 {
        self.underlay_config.width
    }
    pub fn underlay_height(&self) -> f32 {
        self.underlay_config.height
    }
    pub fn spawn_random_node(&mut self) -> Entity {
        self.world
            .spawn(random_node(&self.underlay_config, &mut self.rng))
    }
    pub fn despawn_most_crowded_node(&mut self) -> Result<(), String> {
        if let Some(node) = self.most_crowded_node() {
            self.world.despawn(node).unwrap();
            Ok(())
        } else {
            Err("No nodes left to despawn".to_string())
        }
    }
    pub fn pick_random_node(&mut self) -> Option<Entity> {
        self.all_nodes().choose(&mut self.rng).copied()
    }
    pub fn pick_random_other_node(&mut self, node: Entity) -> Option<Entity> {
        self.all_other_nodes(node).choose(&mut self.rng).copied()
    }
    pub fn all_nodes(&mut self) -> Vec<Entity> {
        self.world
            .query_mut::<&UnderlayNodeName>()
            .into_iter()
            .map(|(id, _)| id)
            .collect()
    }
    pub fn all_other_nodes(&mut self, node: Entity) -> Vec<Entity> {
        self.world
            .query_mut::<&UnderlayNodeName>()
            .into_iter()
            .map(|(id, _)| id)
            .filter(|id| *id != node)
            .collect()
    }
    pub fn send_message<P: Payload>(&mut self, source: Entity, dest: Entity, payload: P) -> Entity {
        let start_time = self.time.now();
        self.spawn_and_schedule_message(source, dest, start_time, payload)
    }
    pub fn send_messages<P: Payload>(
        &mut self,
        source: Entity,
        dest: Entity,
        payloads: impl IntoIterator<Item = P>,
    ) -> Vec<Entity> {
        let per_message_delay = SimSeconds::from(0.001);
        let mut start_time = self.time.now();
        let mut message_entities = vec![];
        for payload in payloads.into_iter() {
            let message_entity = self.spawn_and_schedule_message(source, dest, start_time, payload);
            message_entities.push(message_entity);
            start_time += per_message_delay;
        }
        message_entities
    }
    /// Warning: Current implementation ist not very efficient!
    fn most_crowded_node(&mut self) -> Option<Entity> {
        let all_nodes: Vec<(Entity, UnderlayPosition)> = self
            .world
            .query_mut::<(&UnderlayNodeName, &UnderlayPosition)>()
            .into_iter()
            .map(|(id, (_, &position))| (id, position))
            .collect();

        all_nodes
            .iter()
            .map(|&(node, position)| {
                (
                    node,
                    all_nodes
                        .iter()
                        .filter_map(|&(other_node, other_position)| {
                            (other_node != node)
                                .then(|| 1. / UnderlayPosition::distance(position, other_position))
                        })
                        .sum::<f32>(),
                )
            })
            .max_by_key(|(_, crowdedness_score)| OrderedFloat(*crowdedness_score))
            .map(|(node, _)| node)
    }
    fn spawn_and_schedule_message<P: Payload>(
        &mut self,
        source: Entity,
        dest: Entity,
        start_time: SimSeconds,
        payload: P,
    ) -> Entity {
        let (arrival_time, message_entity) =
            self.spawn_message_entity(source, dest, start_time, payload);
        self.schedule_message(source, dest, message_entity, arrival_time);
        message_entity
    }
    fn spawn_message_entity<P: Payload>(
        &mut self,
        source: Entity,
        dest: Entity,
        start_time: SimSeconds,
        payload: P,
    ) -> (OrderedFloat<f64>, Entity) {
        let trajectory = UnderlayLine::from_nodes(&self.world, source, dest);
        let flight_duration = f64::from(trajectory.length()) / self.underlay_config.message_speed;
        let end_time = start_time + flight_duration;
        let message_entity = self.world.spawn((
            UnderlayMessage { source, dest },
            TimeSpan {
                start: start_time,
                end: end_time,
            },
            trajectory,
            payload,
        ));
        (end_time, message_entity)
    }
    fn schedule_message(
        &mut self,
        source: Entity,
        dest: Entity,
        message_entity: Entity,
        arrival_time: OrderedFloat<f64>,
    ) {
        self.schedule_now(Event::Node(source, NodeEvent::MessageSent(message_entity)));
        self.schedule_at(
            arrival_time,
            Event::Node(dest, NodeEvent::MessageArrived(message_entity)),
        );
    }
}

fn random_node(
    underlay_config: &UnderlayConfig,
    rng: &mut impl Rng,
) -> (UnderlayNodeName, UnderlayPosition) {
    let name = format!("n{:#04}", rng.gen_range(0..10_000));
    (
        UnderlayNodeName(name),
        UnderlayPosition {
            x: rng.gen_range(0f32..underlay_config.width),
            y: rng.gen_range(0f32..underlay_config.height),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn send_random_node_spawns_node() {
        let mut sim = Simulation::new();
        let node_entity = sim.spawn_random_node();
        assert!(sim.world.get::<UnderlayNodeName>(node_entity).is_ok());
        assert!(sim.world.get::<UnderlayPosition>(node_entity).is_ok());
    }

    #[wasm_bindgen_test]
    fn send_message_creates_helper_fields() {
        let mut sim = Simulation::new();
        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        let message_entity = sim.send_message(node1, node2, ());
        assert!(sim.world.get::<UnderlayLine>(message_entity).is_ok());
        assert!(sim.world.get::<TimeSpan>(message_entity).is_ok());
    }

    #[wasm_bindgen_test]
    fn send_message_sets_payload() {
        let mut sim = Simulation::new();
        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        let payload = "test".to_string();
        let message_entity = sim.send_message(node1, node2, payload.clone());

        let expected = payload;
        let actual = sim
            .world
            .query_one::<&String>(message_entity)
            .unwrap()
            .get()
            .unwrap()
            .clone();
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn most_crowded_node_in_line_is_middle_node() {
        let mut sim = Simulation::new();
        let _node1 = sim.world.spawn((
            UnderlayNodeName("node1".to_string()),
            UnderlayPosition { x: 10., y: 10. },
        ));
        let node2 = sim.world.spawn((
            UnderlayNodeName("node2".to_string()),
            UnderlayPosition { x: 20., y: 10. },
        ));
        let _node3 = sim.world.spawn((
            UnderlayNodeName("node3".to_string()),
            UnderlayPosition { x: 30., y: 10. },
        ));

        let expected = Some(node2);
        let actual = sim.most_crowded_node();
        assert_eq!(expected, actual);
    }
}
