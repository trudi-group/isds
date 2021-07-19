#![allow(clippy::cast_possible_truncation)]
use super::*;

pub fn update_animated_objects(world: &mut World, sim_time: SimSeconds) {
    let mut query = <(&UnderlayLine, &TimeSpan, &mut UnderlayPosition)>::query();

    for (path, time_span, position) in query.iter_mut(world) {
        let progress =
            ((sim_time - time_span.start) / (time_span.end - time_span.start)).into_inner() as f32;
        // clippy said that `mul_add` could be faster...
        position.x = (path.end.x - path.start.x).mul_add(progress, path.start.x);
        position.y = (path.end.y - path.start.y).mul_add(progress, path.start.y);
    }
}
