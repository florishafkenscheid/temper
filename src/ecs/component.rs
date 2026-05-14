use std::any::{TypeId, type_name};

pub trait Component: 'static {}

impl<T: 'static> Component for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(TypeId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStorage {
    Table,
    Sparse,
}

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    id: ComponentId,
    name: &'static str,
    storage: ComponentStorage,
}

#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    components: Vec<ComponentInfo>,
}

impl ComponentRegistry {
    pub fn register<T: Component>(&mut self) -> ComponentId {
        self.register_with_storage::<T>(ComponentStorage::Table)
    }

    pub fn register_with_storage<T: Component>(
        &mut self,
        storage: ComponentStorage,
    ) -> ComponentId {
        let id = ComponentId(TypeId::of::<T>());

        if self.components.iter().any(|info| info.id == id) {
            return id;
        }

        self.components.push(ComponentInfo {
            id,
            name: type_name::<T>(),
            storage,
        });

        id
    }
}
