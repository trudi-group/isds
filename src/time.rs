pub use ordered_float::OrderedFloat;

pub type RealSeconds = f64;
pub type SimSeconds = OrderedFloat<f64>;

pub struct Time {
    pub speed_factor: f64,
    sim_time: SimSeconds,
    pub paused: bool,
}
impl Time {
    pub const fn new(speed_factor: f64) -> Self {
        Self {
            speed_factor,
            sim_time: OrderedFloat(0.),
            paused: false,
        }
    }
    pub const fn sim_time(&self) -> SimSeconds {
        self.sim_time
    }
    pub fn advance_sim_time_by(&mut self, elapsed_real_time: RealSeconds) -> Option<SimSeconds> {
        if self.paused {
            return None;
        }
        let diff = OrderedFloat(elapsed_real_time * self.speed_factor);
        if diff > OrderedFloat(0.) {
            self.sim_time += diff;
            Some(diff)
        } else {
            None
        }
    }
    pub fn toggle_paused(&mut self) {
        self.paused = !self.paused;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn timing_realtime() {
        let mut time = Time::new(1.);
        assert_eq!(time.sim_time(), 0.);
        time.advance_sim_time_by(10.);
        assert_eq!(time.sim_time(), 10.);
    }

    #[wasm_bindgen_test]
    fn timing_halftime() {
        let mut time = Time::new(0.5);
        assert_eq!(time.sim_time(), 0.);
        time.advance_sim_time_by(10.);
        assert_eq!(time.sim_time(), 5.);
    }

    #[wasm_bindgen_test]
    fn timing_default_and_speed_change() {
        let mut time = Time::new(10.);
        assert_eq!(time.sim_time(), 0.);
        time.advance_sim_time_by(10.);
        assert_eq!(time.sim_time(), 100.);
        time.speed_factor = 1.;
        time.advance_sim_time_by(10.);
        assert_eq!(time.sim_time(), 110.);
    }

    #[wasm_bindgen_test]
    fn timing_pause_resume() {
        let mut time = Time::new(0.5);
        assert_eq!(time.sim_time(), 0.);
        time.advance_sim_time_by(10.);
        assert_eq!(time.sim_time(), 5.);
        time.toggle_paused();
        time.advance_sim_time_by(10.);
        assert_eq!(time.sim_time(), 5.);
        time.toggle_paused();
        time.advance_sim_time_by(10.);
        assert_eq!(time.sim_time(), 10.);
    }
}
