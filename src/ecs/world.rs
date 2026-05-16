use std::collections::HashMap;

use crate::ecs::{
    component::ComponentRegistry,
    entity::{Entity, EntityAllocator},
    storage::table::{TableComponentKey, TableComponentValue, TableEntityLocation, TableStorage},
};

const DEFAULT_CHUNK_CAPACITY: usize = 1024;

pub struct World {
    entities: EntityAllocator,
    components: ComponentRegistry,
    table_storage: TableStorage,
    locations: HashMap<Entity, TableEntityLocation>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            components: ComponentRegistry::default(),
            table_storage: TableStorage::new(DEFAULT_CHUNK_CAPACITY),
            locations: HashMap::new(),
        }
    }

    pub(crate) fn spawn_table(&mut self, components: Vec<TableComponentValue>) -> Entity {
        let keys = components
            .iter()
            .map(|component| {
                let id = self
                    .components
                    .register_table_id(component.id(), component.name());

                let order = self
                    .components
                    .order(id)
                    .expect("registered component should have an order");

                TableComponentKey::new(id, order)
            })
            .collect::<Vec<_>>();

        let entity = self.entities.spawn();
        let location = self.table_storage.insert(entity, components, keys);
        self.locations.insert(entity, location);
        entity
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.alive_count()
    }

    #[must_use]
    pub fn table_entity_count(&self) -> usize {
        self.table_storage.len()
    }

    #[must_use]
    pub fn archetype_count(&self) -> usize {
        self.table_storage.archetype_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::storage::table::TableComponentValue;

    #[derive(Debug, PartialEq)]
    struct Position(i32);

    #[derive(Debug, PartialEq)]
    struct Velocity(i32);

    #[test]
    fn spawn_table_creates_alive_entity() {
        let mut world = World::new();

        let entity = world.spawn_table(vec![
            TableComponentValue::new(Position(10)),
            TableComponentValue::new(Velocity(1)),
        ]);

        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 1);
    }

    #[test]
    fn same_component_set_reuses_archetype() {
        let mut world = World::new();

        world.spawn_table(vec![
            TableComponentValue::new(Position(10)),
            TableComponentValue::new(Velocity(1)),
        ]);

        world.spawn_table(vec![
            TableComponentValue::new(Position(20)),
            TableComponentValue::new(Velocity(2)),
        ]);

        assert_eq!(world.archetype_count(), 1);
        assert_eq!(world.table_entity_count(), 2);
    }

    #[test]
    fn different_component_set_creates_new_archetype() {
        let mut world = World::new();

        world.spawn_table(vec![TableComponentValue::new(Position(10))]);

        world.spawn_table(vec![
            TableComponentValue::new(Position(20)),
            TableComponentValue::new(Velocity(2)),
        ]);

        assert_eq!(world.archetype_count(), 2);
        assert_eq!(world.table_entity_count(), 2);
    }

    #[test]
    fn component_order_does_not_change_archetype_identity() {
        let mut world = World::new();

        world.spawn_table(vec![
            TableComponentValue::new(Position(10)),
            TableComponentValue::new(Velocity(1)),
        ]);

        world.spawn_table(vec![
            TableComponentValue::new(Velocity(2)),
            TableComponentValue::new(Position(20)),
        ]);

        assert_eq!(world.archetype_count(), 1);
    }
}
