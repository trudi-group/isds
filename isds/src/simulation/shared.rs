use super::*;

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct SharedSimulation(Rc<RefCell<Simulation>>);

impl SharedSimulation {
    pub fn new(sim: Simulation) -> Self {
        Self(Rc::new(RefCell::new(sim)))
    }
    pub fn borrow(&self) -> Ref<Simulation> {
        self.0.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<Simulation> {
        self.0.borrow_mut()
    }
}

impl std::fmt::Debug for SharedSimulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedSimulation")
            .field("details", &"hidden")
            .finish()
    }
}

impl PartialEq for SharedSimulation {
    fn eq(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Simulation {
    pub fn into_shared(self) -> SharedSimulation {
        SharedSimulation::new(self)
    }
}
