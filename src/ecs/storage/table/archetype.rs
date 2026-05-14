use crate::ecs::{
    component::{Component, ComponentId},
    entity::Entity,
};

use super::{Chunk, StoredComponent, TableRowLocation};

pub(crate) struct Archetype {
    component_ids: Vec<ComponentId>,
    chunk_capacity: usize,
    chunks: Vec<Chunk>,
    len: usize,
}

impl Archetype {
    #[must_use]
    pub(crate) fn new(mut component_ids: Vec<ComponentId>, chunk_capacity: usize) -> Self {
        // TODO! Add stable internal ordering in ComponentRegistry
        component_ids.sort_by_key(|id| format!("{id:?}"));

        Self {
            component_ids,
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

    #[must_use]
    pub(crate) fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    #[must_use]
    pub(crate) fn component_ids(&self) -> &[ComponentId] {
        &self.component_ids
    }

    pub(crate) fn push_row(
        &mut self,
        entity: Entity,
        components: Vec<(ComponentId, Box<StoredComponent>)>,
    ) -> TableRowLocation {
        if self.chunks.last().is_none_or(Chunk::is_full) {
            self.chunks
                .push(Chunk::new(&self.component_ids, self.chunk_capacity));
        }

        let chunk = self.chunks.len() - 1;
        let row = self.chunks[chunk].push_row(entity, components);
        self.len += 1;

        TableRowLocation { chunk, row }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position(i32);

    #[derive(Debug, PartialEq)]
    struct Velocity(i32);

    #[derive(Debug, PartialEq)]
    struct Extra;

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
        let mut archetype = Archetype::new(vec![position_id, velocity_id], 4);

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
        let mut archetype = Archetype::new(vec![position_id, velocity_id], 2);

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
        let mut archetype = Archetype::new(vec![position_id, velocity_id], 4);

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
        let mut archetype = Archetype::new(vec![position_id, velocity_id], 4);

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
        let mut archetype = Archetype::new(vec![position_id, velocity_id], 4);

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
        let mut archetype = Archetype::new(vec![position_id, velocity_id], 4);

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
