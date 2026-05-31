use std::any::Any;

use crate::ecs::component::{Component, ComponentId};

pub(crate) type StoredComponent = dyn Any;

pub(crate) struct ComponentColumn {
    component_id: ComponentId,
    values: Vec<Box<StoredComponent>>,
}

impl ComponentColumn {
    #[must_use]
    pub(crate) fn new(component_id: ComponentId) -> Self {
        Self {
            component_id,
            values: Vec::new(),
        }
    }

    #[must_use]
    pub(crate) fn component_id(&self) -> ComponentId {
        self.component_id
    }

    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn push(&mut self, value: Box<StoredComponent>) {
        self.values.push(value);
    }

    #[must_use]
    pub(crate) fn get<T: Component>(&self, row: usize) -> Option<&T> {
        self.values.get(row)?.downcast_ref()
    }

    #[must_use]
    pub(crate) fn get_mut<T: Component>(&mut self, row: usize) -> Option<&mut T> {
        self.values.get_mut(row)?.downcast_mut()
    }

    pub(crate) fn iter_mut<T: Component>(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.values.iter_mut().map(|value| {
            value
                .downcast_mut()
                .expect("component column should contain requested type")
        })
    }

    pub(crate) fn swap_remove(&mut self, row: usize) -> Box<StoredComponent> {
        self.values.swap_remove(row)
    }
}
