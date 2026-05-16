use std::any::type_name;

use crate::ecs::component::{Component, ComponentId, ComponentOrder};

use super::StoredComponent;

pub(crate) struct TableComponentValue {
    id: ComponentId,
    name: &'static str,
    value: Box<StoredComponent>,
}

impl TableComponentValue {
    #[must_use]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TableComponentKey {
    id: ComponentId,
    order: ComponentOrder,
}

impl TableComponentKey {
    #[must_use]
    pub(crate) fn new(id: ComponentId, order: ComponentOrder) -> Self {
        Self { id, order }
    }

    pub(crate) fn id(self) -> ComponentId {
        self.id
    }

    pub(crate) fn order(self) -> ComponentOrder {
        self.order
    }
}
