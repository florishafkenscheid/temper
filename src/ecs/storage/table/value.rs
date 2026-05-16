use std::any::type_name;

use crate::ecs::component::{Component, ComponentId};

use super::StoredComponent;

pub(crate) struct TableComponentValue {
    id: ComponentId,
    name: &'static str,
    value: Box<StoredComponent>,
}

impl TableComponentValue {
    pub(crate) fn new<T: Component>(value: T) -> Self {
        Self {
            id: ComponentId::of::<T>(),
            name: type_name::<T>(),
            value: Box::new(value),
        }
    }

    pub(crate) fn id(&self) -> ComponentId {
        self.id
    }

    pub(crate) fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn into_parts(self) -> (ComponentId, Box<StoredComponent>) {
        (self.id, self.value)
    }
}
