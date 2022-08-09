use super::*;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Highlight(Rc<RefCell<Option<Entity>>>);

impl Highlight {
    /// Whether this is the highlighed entity.
    pub fn is(&self, entity: Entity) -> bool {
        if let Ok(highlighted_entity) = self.0.try_borrow() {
            highlighted_entity.map_or(false, |e| e == entity)
        } else {
            log!("Error borrowing the highlight!");
            false
        }
    }
    pub fn set_highlight(&self, entity: Entity) {
        if let Ok(mut highlighted_entity) = self.0.try_borrow_mut() {
            *highlighted_entity = Some(entity);
        } else {
            log!("Error borrowing the highlight!");
        }
    }
    pub fn reset_highlight(&self) {
        if let Ok(mut highlighted_entity) = self.0.try_borrow_mut() {
            *highlighted_entity = None;
        } else {
            log!("Error borrowing the highlight!");
        }
    }
}
