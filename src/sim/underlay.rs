use super::*;

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

pub struct UnderlayMessage {
    pub source: Entity,
    pub dest: Entity,
    // TODO: payload: ProtocolMessage
}

impl Simulator {
    pub fn spawn_random_node(&mut self, world: &mut World) -> Entity {
        world.spawn(random_node(&mut self.rng))
    }
    pub fn spawn_message_between_random_nodes(
        &mut self,
        world: &mut World,
        start_time: SimSeconds,
    ) -> Result<Entity, &str> {
        let node_ents: Vec<Entity> = world
            .query_mut::<&UnderlayNodeName>()
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        if node_ents.len() < 2 {
            Err("Not enough nodes around.")
        } else {
            let selected_node_ids: Vec<Entity> = node_ents
                .choose_multiple(&mut self.rng, 2)
                .copied()
                .collect();
            let source = selected_node_ids[0];
            let dest = selected_node_ids[1];
            Ok(self.spawn_message(world, start_time, source, dest))
        }
    }
    pub fn spawn_message_to_random_node(
        &mut self,
        world: &mut World,
        start_time: SimSeconds,
        source: Entity,
    ) -> Result<Entity, &str> {
        let node_ents: Vec<Entity> = world
            .query_mut::<&UnderlayNodeName>()
            .into_iter()
            .map(|(id, _)| id)
            .filter(|id| *id != source)
            .collect();
        if let Some(&dest) = node_ents.choose(&mut self.rng) {
            Ok(self.spawn_message(world, start_time, source, dest))
        } else {
            Err("Couldn't find a suitable message destination. Not enough nodes around?")
        }
    }
    pub fn spawn_message(
        &mut self,
        world: &mut World,
        start_time: SimSeconds,
        source: Entity,
        dest: Entity,
    ) -> Entity {
        let trajectory = UnderlayLine::from_nodes(world, source, dest);

        let flight_duration = f64::from(trajectory.length()) / FLIGHT_PER_SECOND;
        let end_time = start_time + flight_duration;

        let position = trajectory.start;

        let message_entity = world.spawn((
            UnderlayMessage { source, dest },
            TimeSpan {
                start: start_time,
                end: end_time,
            },
            trajectory,
            position,
        ));
        self.log(
            start_time,
            format!(
                "{}: Sending a message to {}",
                name(world, source),
                name(world, dest),
            ),
        );
        self.schedule(end_time, SimEvent::MessageArrived(message_entity));
        message_entity
    }
}

fn random_node(rng: &mut impl Rng) -> (UnderlayNodeName, UnderlayPosition) {
    let name = format!("node{:#04}", rng.gen_range(0..10_000));
    let buffer_zone = 10.;
    (
        UnderlayNodeName(name),
        UnderlayPosition {
            x: rng.gen_range(buffer_zone..=(NET_MAX_X - buffer_zone)),
            y: rng.gen_range(buffer_zone..=(NET_MAX_Y - buffer_zone)),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn spawn_random_node_spawns_node() {
        let mut world = World::default();
        let mut simulator = Simulator::new();
        let node_entity = simulator.spawn_random_node(&mut world);
        assert!(world.get::<UnderlayNodeName>(node_entity).is_ok());
        assert!(world.get::<UnderlayPosition>(node_entity).is_ok());
    }

    #[wasm_bindgen_test]
    fn spawn_message_creates_helper_fields() {
        let mut world = World::default();
        let mut simulator = Simulator::new();
        let node1 = simulator.spawn_random_node(&mut world);
        let node2 = simulator.spawn_random_node(&mut world);
        let message_entity =
            simulator.spawn_message(&mut world, SimSeconds::default(), node1, node2);
        assert!(world.get::<UnderlayLine>(message_entity).is_ok());
        assert!(world.get::<TimeSpan>(message_entity).is_ok());
    }
}
