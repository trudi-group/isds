use super::*;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Highlight {
    hovered: Rc<RefCell<Option<Entity>>>,
    selected: Rc<RefCell<Option<Entity>>>,
}

impl Highlight {
    #![allow(clippy::unnecessary_unwrap)]
    /// Whether this is a highlighed entity.
    pub fn is(&self, entity: Entity) -> bool {
        let hovered = self.hovered.try_borrow();
        let selected = self.selected.try_borrow();

        if hovered.is_err() || selected.is_err() {
            log!("Error borrowing the highlight!");
            false
        } else {
            hovered
                .unwrap()
                .map_or(false, |hovered_entity| hovered_entity == entity)
                || selected
                    .unwrap()
                    .map_or(false, |selected_entity| selected_entity == entity)
        }
    }
    pub fn set_hover(&self, entity: Entity) {
        if let Ok(mut highlighted_entity) = self.hovered.try_borrow_mut() {
            *highlighted_entity = Some(entity);
        } else {
            log!("Error borrowing the highlight!");
        }
    }
    pub fn set_select(&self, entity: Entity) {
        if let Ok(mut highlighted_entity) = self.selected.try_borrow_mut() {
            *highlighted_entity = Some(entity);
        } else {
            log!("Error borrowing the highlight!");
        }
    }
    pub fn reset_hover(&self) {
        if let Ok(mut highlighted_entity) = self.hovered.try_borrow_mut() {
            *highlighted_entity = None;
        } else {
            log!("Error borrowing the highlight!");
        }
    }
    pub fn reset_select(&self) {
        if let Ok(mut highlighted_entity) = self.selected.try_borrow_mut() {
            *highlighted_entity = None;
        } else {
            log!("Error borrowing the highlight!");
        }
    }
    pub fn toggle_select(&self, entity: Entity) {
        if let Ok(mut highlighted_entity) = self.selected.try_borrow_mut() {
            if *highlighted_entity == Some(entity) {
                *highlighted_entity = None;
            } else {
                *highlighted_entity = Some(entity);
            }
        } else {
            log!("Error borrowing the highlight!");
        }
    }
}
