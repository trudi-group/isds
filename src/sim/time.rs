pub use ordered_float::OrderedFloat;

pub type RealSeconds = f64;
pub type SimSeconds = OrderedFloat<f64>;

pub struct Time {
    speed_factor: f64,
    sim_time: SimSeconds,
    paused: bool,
}
impl Time {
    pub const fn new(speed_factor: f64) -> Self {
        Self {
            speed_factor,
            sim_time: OrderedFloat(0.),
            paused: false,
        }
    }
    pub const fn now(&self) -> SimSeconds {
        self.sim_time
    }
    pub const fn speed(&self) -> f64 {
        self.speed_factor
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn after(&self, elapsed_real_time: RealSeconds) -> SimSeconds {
        if self.paused {
            self.sim_time
        } else {
            let diff = OrderedFloat(elapsed_real_time * self.speed_factor);
            self.sim_time + diff
        }
    }
    pub fn advance_sim_time_to(&mut self, new_now: SimSeconds) {
        debug_assert!(new_now >= self.sim_time);
        self.sim_time = new_now;
    }
    pub fn set_speed(&mut self, speed: f64) {
        self.speed_factor = speed;
    }
    pub fn toggle_paused(&mut self) {
        self.paused = !self.paused;
    }
}

pub struct TimeSpan {
    pub start: SimSeconds,
    pub end: SimSeconds,
}
impl TimeSpan {
    pub fn progress(&self, time_now: SimSeconds) -> f64 {
        ((time_now - self.start) / (self.end - self.start)).into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn timing_realtime() {
        let time = Time::new(1.);
        assert_eq!(time.now(), 0.);
        assert_eq!(time.after(10.), 10.);
    }

    #[wasm_bindgen_test]
    fn timing_halftime() {
        let time = Time::new(0.5);
        assert_eq!(time.now(), 0.);
        assert_eq!(time.after(10.), 5.);
    }

    #[wasm_bindgen_test]
    fn timing_default_and_speed_change() {
        let mut time = Time::new(10.);
        assert_eq!(time.now(), 0.);
        assert_eq!(time.after(10.), 100.);
        time.advance_sim_time_to((100.).into());
        assert_eq!(time.now(), 100.);
        time.speed_factor = 1.;
        assert_eq!(time.after(10.), 110.);
    }

    #[wasm_bindgen_test]
    fn timing_pause_resume() {
        let mut time = Time::new(0.5);
        assert_eq!(time.now(), 0.);
        time.toggle_paused();
        assert_eq!(time.after(10.), 0.);
        time.toggle_paused();
        assert_eq!(time.after(10.), 5.);
    }
}
