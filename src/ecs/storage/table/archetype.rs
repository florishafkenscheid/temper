use crate::ecs::{
    component::{Component, ComponentId},
    entity::Entity,
    storage::table::{TableComponentKey, chunk::RemovedChunkRow},
};

use super::{Chunk, StoredComponent, TableRowLocation};

pub(crate) struct Archetype {
    components: Vec<TableComponentKey>,
    chunk_capacity: usize,
    chunks: Vec<Chunk>,
    len: usize,
}

impl Archetype {
    #[must_use]
    pub(crate) fn new(mut components: Vec<TableComponentKey>, chunk_capacity: usize) -> Self {
        components.sort_by_key(|component| component.order());

        Self {
            components,
            chunk_capacity,
            chunks: Vec::new(),
            len: 0,
        }
    }

    #[must_use]
    pub(crate) fn get<T: Component>(
        &self,
        component_id: ComponentId,
        location: TableRowLocation,
    ) -> Option<&T> {
        self.chunks
            .get(location.chunk)?
            .get(component_id, location.row)
    }

    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn chunk_capacity(&self) -> usize {
        self.chunk_capacity
    }

    #[must_use]
    pub(crate) fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub(crate) fn components(&self) -> &[TableComponentKey] {
        &self.components
    }

    pub(crate) fn component_ids(&self) -> Vec<ComponentId> {
        self.components
            .iter()
            .map(|component| component.id())
            .collect()
    }

    pub(crate) fn contains_component(&self, component_id: ComponentId) -> bool {
        self.components
            .iter()
            .any(|component| component.id() == component_id)
    }

    pub(crate) fn component_keys_without(
        &self,
        component_id: ComponentId,
    ) -> Vec<TableComponentKey> {
        self.components
            .iter()
            .copied()
            .filter(|component| component.id() != component_id)
            .collect()
    }

    pub(crate) fn push_row(
        &mut self,
        entity: Entity,
        components: Vec<(ComponentId, Box<StoredComponent>)>,
    ) -> TableRowLocation {
        if self.chunks.last().is_none_or(Chunk::is_full) {
            self.chunks
                .push(Chunk::new(&self.component_ids(), self.chunk_capacity()));
        }

        let chunk = self.chunks.len() - 1;
        let row = self.chunks[chunk].push_row(entity, components);
        self.len += 1;

        TableRowLocation { chunk, row }
    }

    pub(crate) fn remove_row(&mut self, location: TableRowLocation) -> Option<Entity> {
        self.take_row(location).moved_entity
    }

    pub(crate) fn take_row(&mut self, location: TableRowLocation) -> RemovedChunkRow {
        let chunk = self
            .chunks
            .get_mut(location.chunk)
            .expect("table row location should reference an existing chunk");

        let row = chunk
            .take_row(location.row)
            .expect("table row location should reference an existing row");

        self.len -= 1;
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::ComponentOrder;

    #[derive(Debug, PartialEq)]
    struct Position(i32);

    #[derive(Debug, PartialEq)]
    struct Velocity(i32);

    #[derive(Debug, PartialEq)]
    struct Extra;

    fn key(component_id: ComponentId, order: usize) -> TableComponentKey {
        TableComponentKey::new(component_id, ComponentOrder::new(order))
    }

    fn row(
        position_id: ComponentId,
        velocity_id: ComponentId,
        position: i32,
        velocity: i32,
    ) -> Vec<(ComponentId, Box<StoredComponent>)> {
        vec![
            (position_id, Box::new(Position(position))),
            (velocity_id, Box::new(Velocity(velocity))),
        ]
    }

    #[test]
    fn pushed_rows_are_dense_within_chunk() {
        let position_id = ComponentId::of::<Position>();
        let velocity_id = ComponentId::of::<Velocity>();
        let mut archetype = Archetype::new(vec![key(position_id, 0), key(velocity_id, 1)], 4);

        let first = archetype.push_row(Entity::new(0, 0), row(position_id, velocity_id, 10, 1));
        let second = archetype.push_row(Entity::new(1, 0), row(position_id, velocity_id, 20, 2));

        assert_eq!(first, TableRowLocation { chunk: 0, row: 0 });
        assert_eq!(second, TableRowLocation { chunk: 0, row: 1 });
        assert_eq!(archetype.len(), 2);
        assert_eq!(archetype.chunk_count(), 1);
    }

    #[test]
    fn archetype_splits_rows_across_chunks_at_capacity() {
        let position_id = ComponentId::of::<Position>();
        let velocity_id = ComponentId::of::<Velocity>();
        let mut archetype = Archetype::new(vec![key(position_id, 0), key(velocity_id, 1)], 2);

        let first = archetype.push_row(Entity::new(0, 0), row(position_id, velocity_id, 10, 1));
        let second = archetype.push_row(Entity::new(1, 0), row(position_id, velocity_id, 20, 2));
        let third = archetype.push_row(Entity::new(2, 0), row(position_id, velocity_id, 30, 3));

        assert_eq!(first, TableRowLocation { chunk: 0, row: 0 });
        assert_eq!(second, TableRowLocation { chunk: 0, row: 1 });
        assert_eq!(third, TableRowLocation { chunk: 1, row: 0 });
        assert_eq!(archetype.len(), 3);
        assert_eq!(archetype.chunk_count(), 2);
    }

    #[test]
    fn component_columns_align_by_row() {
        let position_id = ComponentId::of::<Position>();
        let velocity_id = ComponentId::of::<Velocity>();
        let mut archetype = Archetype::new(vec![key(position_id, 0), key(velocity_id, 1)], 4);

        let first = archetype.push_row(Entity::new(0, 0), row(position_id, velocity_id, 10, 1));
        let second = archetype.push_row(Entity::new(1, 0), row(position_id, velocity_id, 20, 2));

        assert_eq!(
            archetype.get::<Position>(position_id, first),
            Some(&Position(10)),
        );
        assert_eq!(
            archetype.get::<Velocity>(velocity_id, first),
            Some(&Velocity(1)),
        );
        assert_eq!(
            archetype.get::<Position>(position_id, second),
            Some(&Position(20)),
        );
        assert_eq!(
            archetype.get::<Velocity>(velocity_id, second),
            Some(&Velocity(2)),
        );
    }

    #[test]
    fn component_order_does_not_need_to_match_column_order() {
        let position_id = ComponentId::of::<Position>();
        let velocity_id = ComponentId::of::<Velocity>();
        let mut archetype = Archetype::new(vec![key(position_id, 0), key(velocity_id, 1)], 4);

        let location = archetype.push_row(
            Entity::new(0, 0),
            vec![
                (velocity_id, Box::new(Velocity(7))),
                (position_id, Box::new(Position(42))),
            ],
        );

        assert_eq!(
            archetype.get::<Position>(position_id, location),
            Some(&Position(42)),
        );
        assert_eq!(
            archetype.get::<Velocity>(velocity_id, location),
            Some(&Velocity(7)),
        );
    }

    #[test]
    #[should_panic(expected = "row is missing component for archetype column")]
    fn push_row_rejects_missing_component_for_archetype() {
        let position_id = ComponentId::of::<Position>();
        let velocity_id = ComponentId::of::<Velocity>();
        let mut archetype = Archetype::new(vec![key(position_id, 0), key(velocity_id, 1)], 4);

        archetype.push_row(
            Entity::new(0, 0),
            vec![(position_id, Box::new(Position(10)))],
        );
    }

    #[test]
    #[should_panic(expected = "row contains components that do not belong to the archetype")]
    fn push_row_rejects_extra_component_for_archetype() {
        let position_id = ComponentId::of::<Position>();
        let velocity_id = ComponentId::of::<Velocity>();
        let extra_id = ComponentId::of::<Extra>();
        let mut archetype = Archetype::new(vec![key(position_id, 0), key(velocity_id, 1)], 4);

        archetype.push_row(
            Entity::new(0, 0),
            vec![
                (position_id, Box::new(Position(10))),
                (velocity_id, Box::new(Velocity(1))),
                (extra_id, Box::new(Extra)),
            ],
        );
    }
}
