use std::any::{TypeId, type_name};

pub trait Component: 'static {}

impl<T: 'static> Component for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(TypeId);

impl ComponentId {
    pub(crate) fn of<T: Component>() -> Self {
        Self(TypeId::of::<T>())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStorage {
    Table,
    Sparse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ComponentOrder(usize);

impl ComponentOrder {
    pub(crate) fn new(index: usize) -> Self {
        Self(index)
    }

    pub(crate) fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    id: ComponentId,
    order: ComponentOrder,
    name: &'static str,
    storage: ComponentStorage,
}

#[derive(Debug, Clone, Default)]
pub struct ComponentRegistry {
    components: Vec<ComponentInfo>,
}

impl ComponentRegistry {
    pub fn register<T: Component>(&mut self) -> ComponentId {
        self.register_with_storage::<T>(ComponentStorage::Table)
    }

    pub(crate) fn register_table_id(&mut self, id: ComponentId, name: &'static str) -> ComponentId {
        if self.components.iter().any(|info| info.id == id) {
            return id;
        }

        let order = ComponentOrder::new(self.components.len());

        self.components.push(ComponentInfo {
            id,
            order,
            name,
            storage: ComponentStorage::Table,
        });

        id
    }

    pub fn register_with_storage<T: Component>(
        &mut self,
        storage: ComponentStorage,
    ) -> ComponentId {
        let id = ComponentId::of::<T>();

        if self.components.iter().any(|info| info.id == id) {
            return id;
        }

        let order = ComponentOrder::new(self.components.len());

        self.components.push(ComponentInfo {
            id,
            order,
            name: type_name::<T>(),
            storage,
        });

        id
    }

    pub(crate) fn order(&self, id: ComponentId) -> Option<ComponentOrder> {
        self.components
            .iter()
            .find(|info| info.id == id)
            .map(|info| info.order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position;
    struct Velocity;

    #[test]
    fn registration_assigns_stable_order() {
        let mut registry = ComponentRegistry::default();

        let position = registry.register::<Position>();
        let velocity = registry.register::<Velocity>();

        assert!(registry.order(position) < registry.order(velocity));
    }

    #[test]
    fn repeated_registration_keeps_original_order() {
        let mut registry = ComponentRegistry::default();

        let first = registry.register::<Position>();
        let first_order = registry.order(first);

        let second = registry.register::<Position>();
        let second_order = registry.order(second);

        assert_eq!(first, second);
        assert_eq!(first_order, second_order);
    }

    #[test]
    fn table_id_registration_uses_next_order() {
        let mut registry = ComponentRegistry::default();

        let position = ComponentId::of::<Position>();
        let velocity = ComponentId::of::<Velocity>();

        registry.register_table_id(position, "Position");
        registry.register_table_id(velocity, "Velocity");

        assert_eq!(registry.order(position).map(ComponentOrder::index), Some(0));
        assert_eq!(registry.order(velocity).map(ComponentOrder::index), Some(1));
    }
}
