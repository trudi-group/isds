use super::*;

// TODO perhaps make this use the real logger interface to be able to decouple Simulator from seed
// one day?

pub struct Logger {
    log: VecDeque<(SimSeconds, String)>,
}
impl Logger {
    pub fn new() -> Self {
        Self {
            log: VecDeque::new(),
        }
    }
    pub fn log(&mut self, sim_time: SimSeconds, message: String) {
        // seed::log!(format!("{}: {}", sim_time, message));
        self.log.push_front((sim_time, message));
        self.log.truncate(12);
    }
    pub fn entries(&self) -> impl DoubleEndedIterator<Item = &(SimSeconds, String)> {
        self.log.iter()
    }
}
impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
