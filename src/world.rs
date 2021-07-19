use super::SimSeconds;
use legion::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnderlayNodeName(pub(crate) String);

#[derive(Debug, Copy, Clone)]
pub struct UnderlayPosition {
    pub x: f32,
    pub y: f32,
}
impl UnderlayPosition {
    pub fn distance(point1: Self, point2: Self) -> f32 {
        let x = (point1.x - point2.x).abs();
        let y = (point1.y - point2.y).abs();
        x.hypot(y)
    }
}

pub struct UnderlayLine {
    pub start: UnderlayPosition,
    pub end: UnderlayPosition,
}

pub struct UnderlayMessage {
    pub source: Entity,
    pub dest: Entity,
    // TODO: payload: ProtocolMessage
}

pub struct TimeSpan {
    pub start: SimSeconds,
    pub end: SimSeconds,
}

pub fn name(world: &World, entity: Entity) -> &str {
    if let Ok(entry) = world.entry_ref(entity) {
        if let Ok(node_id) = entry.into_component::<UnderlayNodeName>() {
            &node_id.0
        } else {
            "UNNAMEABLE"
        }
    } else {
        "INEXISTING"
    }
}
