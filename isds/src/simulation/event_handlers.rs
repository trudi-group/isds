use super::*;

pub trait EventHandler: AsAny {
    fn handle_event(&mut self, sim: &mut Simulation, event: Event) -> Result<(), Box<dyn Error>>;
}

// we need this for enabling downcasting
pub trait AsAny: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct EventHandlers(Vec<Box<dyn EventHandler>>);
impl EventHandlers {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn add(&mut self, event_handler: impl EventHandler + 'static) -> usize {
        self.0.push(Box::new(event_handler));
        self.0.len() - 1
    }
    pub fn get<T>(&self, handler_index: usize) -> Option<&T>
    where
        T: EventHandler,
    {
        self.0
            .get(handler_index)
            .and_then(|handler| (**handler).as_any().downcast_ref::<T>())
    }
    pub fn get_mut<T>(&mut self, handler_index: usize) -> Option<&mut T>
    where
        T: EventHandler,
    {
        self.0
            .get_mut(handler_index)
            .and_then(|handler| (**handler).as_any_mut().downcast_mut::<T>())
    }
    pub(crate) fn handle_event(
        &mut self,
        sim: &mut Simulation,
        event: Event,
    ) -> Result<(), Box<dyn Error>> {
        for handler in self.0.iter_mut() {
            handler.handle_event(sim, event)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    struct TestHandler(bool);
    impl EventHandler for TestHandler {
        fn handle_event(&mut self, _: &mut Simulation, _: Event) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }

    #[wasm_bindgen_test]
    fn get_event_handler() {
        let mut handlers = EventHandlers::new();
        let i = handlers.add(TestHandler(false));

        let expected = Some(TestHandler(false));
        let actual = handlers.get::<TestHandler>(i).copied();

        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn mut_event_handler() {
        let mut handlers = EventHandlers::new();
        let i = handlers.add(TestHandler(false));

        handlers.get_mut::<TestHandler>(i).unwrap().0 = true;

        let expected = Some(TestHandler(true));
        let actual = handlers.get::<TestHandler>(i).copied();

        assert_eq!(expected, actual);
    }
}
