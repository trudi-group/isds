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
    pub fn send_message<P: hecs::Component>(
        &mut self,
        source: Entity,
        dest: Entity,
        payload: P,
    ) -> Entity {
        let trajectory = UnderlayLine::from_nodes(&self.world, source, dest);
        let flight_duration = f64::from(trajectory.length()) / self.underlay_config.message_speed;

        let start_time = self.time.now();
        let end_time = start_time + flight_duration;

        let position = trajectory.start; // FIXME try perf without me

        let message_entity = self.world.spawn((
            UnderlayMessage { source, dest },
            TimeSpan {
                start: start_time,
                end: end_time,
            },
            trajectory,
            position,
            payload,
        ));
        self.log(format!(
            "{}: Sending a message to {}",
            self.name(source),
            self.name(dest),
        ));
        self.schedule_at(
            end_time,
            Event::Node(dest, NodeEvent::MessageArrived(message_entity)),
        );
        message_entity
    }
}

fn random_node(
    underlay_config: &UnderlayConfig,
    rng: &mut impl Rng,
) -> (UnderlayNodeName, UnderlayPosition) {
    let name = format!("node{:#04}", rng.gen_range(0..10_000));
    let buffer_zone = 10.;
    (
        UnderlayNodeName(name),
        UnderlayPosition {
            x: rng.gen_range(buffer_zone..=(underlay_config.width - buffer_zone)),
            y: rng.gen_range(buffer_zone..=(underlay_config.height - buffer_zone)),
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
}
