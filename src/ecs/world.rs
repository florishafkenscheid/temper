use std::collections::HashMap;

use crate::ecs::{
    bundle::Bundle,
    component::{Component, ComponentId, ComponentRegistry},
    entity::{Entity, EntityAllocator},
    query::{QueryItem, QueryItem2},
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

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let location = self.locations.get(&entity).copied()?;

        self.table_storage.get(location, ComponentId::of::<T>())
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let location = self.locations.get(&entity).copied()?;

        self.table_storage.get_mut(location, ComponentId::of::<T>())
    }

    pub fn query<T: Component>(&self) -> Vec<QueryItem<'_, T>> {
        self.table_storage.query(ComponentId::of::<T>())
    }

    // TODO refactor
    pub fn query2<A: Component, B: Component>(&self) -> Vec<QueryItem2<'_, A, B>> {
        self.table_storage
            .query2(ComponentId::of::<A>(), ComponentId::of::<B>())
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        self.spawn_table(bundle.into_bundle().into_table_components())
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

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let Some(location) = self.locations.remove(&entity) else {
            return self.entities.despawn(entity);
        };

        if let Some(moved_entity) = self.table_storage.remove(location) {
            self.locations.insert(moved_entity, location);
        }

        self.entities.despawn(entity)
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        let Some(location) = self.locations.get(&entity).copied() else {
            return false;
        };

        let Some(removal) = self
            .table_storage
            .remove_component(location, ComponentId::of::<T>())
        else {
            return false;
        };

        if let Some(moved_entity) = removal.moved_entity {
            self.locations.insert(moved_entity, location);
        }

        match removal.new_location {
            Some(new_location) => {
                self.locations.insert(removal.removed_entity, new_location);
            }
            None => {
                self.locations.remove(&removal.removed_entity);
            }
        }

        true
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

    #[derive(Debug, PartialEq)]
    struct Health(i32);

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

    #[test]
    fn spawn_accepts_typed_component_bundle() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 1);
        assert_eq!(world.archetype_count(), 1);
    }

    #[test]
    fn typed_spawn_reuses_archetype_for_same_bundle_types() {
        let mut world = World::new();

        world.spawn((Position(10), Velocity(1)));
        world.spawn((Position(20), Velocity(2)));

        assert_eq!(world.archetype_count(), 1);
        assert_eq!(world.table_entity_count(), 2);
    }

    #[test]
    fn typed_spawn_supports_single_component_tuple() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert!(world.is_alive(entity));
        assert_eq!(world.archetype_count(), 1);
    }

    #[test]
    fn despawn_removes_alive_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.despawn(entity));
        assert!(!world.is_alive(entity));
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.table_entity_count(), 0);
    }

    #[test]
    fn despawn_rejects_stale_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        assert!(world.despawn(entity));

        assert!(!world.despawn(entity));
    }

    #[test]
    fn despawn_repairs_moved_entity_location() {
        let mut world = World::new();

        let first = world.spawn((Position(10), Velocity(1)));
        let second = world.spawn((Position(20), Velocity(2)));

        assert!(world.despawn(first));
        assert!(world.is_alive(second));

        let second_location = world
            .locations
            .get(&second)
            .expect("moved entity should still have a table location");

        assert_eq!(second_location.row.chunk, 0);
        assert_eq!(second_location.row.row, 0);

        assert!(world.despawn(second));
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.table_entity_count(), 0);
    }

    #[test]
    fn despawn_unknown_entity_returns_false() {
        let mut world = World::new();

        assert!(!world.despawn(Entity::new(123, 0)));
    }

    #[test]
    fn remove_component_moves_entity_to_smaller_archetype() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.remove_component::<Velocity>(entity));
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 1);
        assert_eq!(world.archetype_count(), 2);
    }

    #[test]
    fn remove_component_rejects_missing_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert!(!world.remove_component::<Velocity>(entity));
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 1);
    }

    #[test]
    fn remove_component_rejects_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));
        assert!(world.despawn(entity));

        assert!(!world.remove_component::<Velocity>(entity));
    }

    #[test]
    fn remove_last_table_component_keeps_entity_alive_without_table_location() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert!(world.remove_component::<Position>(entity));
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.table_entity_count(), 0);
        assert!(!world.locations.contains_key(&entity));
    }

    #[test]
    fn remove_component_repairs_swap_moved_entity_location() {
        let mut world = World::new();

        let first = world.spawn((Position(10), Velocity(1), Health(100)));
        let second = world.spawn((Position(20), Velocity(2), Health(90)));

        assert!(world.remove_component::<Velocity>(first));

        let second_location = world
            .locations
            .get(&second)
            .expect("swap-moved entity should still have a table location");

        assert_eq!(second_location.row.chunk, 0);
        assert_eq!(second_location.row.row, 0);

        assert!(world.is_alive(first));
        assert!(world.is_alive(second));
        assert_eq!(world.entity_count(), 2);
        assert_eq!(world.table_entity_count(), 2);
    }

    #[test]
    fn get_component_returns_spawned_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
        assert_eq!(world.get_component::<Velocity>(entity), Some(&Velocity(1)));
    }

    #[test]
    fn get_component_returns_none_for_missing_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert_eq!(world.get_component::<Velocity>(entity), None);
    }

    #[test]
    fn get_component_returns_none_for_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        assert!(world.despawn(entity));

        assert_eq!(world.get_component::<Position>(entity), None);
    }

    #[test]
    fn get_component_reflects_component_removal() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.remove_component::<Velocity>(entity));

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(10)));
        assert_eq!(world.get_component::<Velocity>(entity), None);
    }

    #[test]
    fn get_component_survives_swap_moved_location_repair() {
        let mut world = World::new();

        let first = world.spawn((Position(10), Velocity(1)));
        let second = world.spawn((Position(20), Velocity(2)));

        assert!(world.despawn(first));

        assert_eq!(world.get_component::<Position>(second), Some(&Position(20)));
        assert_eq!(world.get_component::<Velocity>(second), Some(&Velocity(2)));
    }

    #[test]
    fn get_component_mut_updates_component_value() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        let position = world
            .get_component_mut::<Position>(entity)
            .expect("entity should have Position");

        position.0 += 5;

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(15)));
    }

    #[test]
    fn get_component_mut_returns_none_for_missing_component() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));

        assert!(world.get_component_mut::<Velocity>(entity).is_none());
    }

    #[test]
    fn get_component_mut_returns_none_for_dead_entity() {
        let mut world = World::new();

        let entity = world.spawn((Position(10),));
        assert!(world.despawn(entity));

        assert!(world.get_component_mut::<Position>(entity).is_none());
    }

    #[test]
    fn get_component_mut_reflects_component_removal() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.remove_component::<Velocity>(entity));

        assert!(world.get_component_mut::<Velocity>(entity).is_none());

        let position = world
            .get_component_mut::<Position>(entity)
            .expect("remaining component should still be mutable");

        position.0 = 20;

        assert_eq!(world.get_component::<Position>(entity), Some(&Position(20)));
    }

    #[test]
    fn query_returns_entities_with_component() {
        let mut world = World::new();

        let first = world.spawn((Position(10),));
        let second = world.spawn((Position(20), Velocity(2)));
        world.spawn((Velocity(3),));

        let results = world.query::<Position>();

        assert_eq!(results.len(), 2);
        assert!(
            results
                .iter()
                .any(|item| item.entity == first && item.component == &Position(10))
        );
        assert!(
            results
                .iter()
                .any(|item| item.entity == second && item.component == &Position(20))
        );
    }

    #[test]
    fn query2_returns_entities_with_both_components() {
        let mut world = World::new();

        world.spawn((Position(10),));
        let matching = world.spawn((Position(20), Velocity(2)));
        world.spawn((Velocity(3),));

        let results = world.query2::<Position, Velocity>();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity, matching);
        assert_eq!(results[0].first, &Position(20));
        assert_eq!(results[0].second, &Velocity(2));
    }

    #[test]
    fn query_ignores_removed_components() {
        let mut world = World::new();

        let entity = world.spawn((Position(10), Velocity(1)));

        assert!(world.remove_component::<Velocity>(entity));

        assert_eq!(world.query::<Position>().len(), 1);
        assert_eq!(world.query::<Velocity>().len(), 0);
        assert_eq!(world.query2::<Position, Velocity>().len(), 0);
    }

    #[test]
    fn query_ignores_despawned_entities() {
        let mut world = World::new();

        let first = world.spawn((Position(10),));
        let second = world.spawn((Position(20),));

        assert!(world.despawn(first));

        let results = world.query::<Position>();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity, second);
        assert_eq!(results[0].component, &Position(20));
    }
}
