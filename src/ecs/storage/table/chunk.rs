use crate::ecs::{
    component::{Component, ComponentId},
    entity::Entity,
};

use super::{ComponentColumn, StoredComponent};

pub(crate) struct RemovedChunkRow {
    pub(crate) entity: Entity,
    pub(crate) components: Vec<(ComponentId, Box<StoredComponent>)>,
    pub(crate) moved_entity: Option<Entity>,
}

pub(crate) struct Chunk {
    capacity: usize,
    entities: Vec<Entity>,
    columns: Vec<ComponentColumn>,
}

impl Chunk {
    #[must_use]
    pub(crate) fn new(component_ids: &[ComponentId], capacity: usize) -> Self {
        Self {
            capacity,
            entities: Vec::with_capacity(capacity),
            columns: component_ids
                .iter()
                .copied()
                .map(ComponentColumn::new)
                .collect(),
        }
    }

    #[must_use]
    pub(crate) fn get<T: Component>(&self, component_id: ComponentId, row: usize) -> Option<&T> {
        self.columns
            .iter()
            .find(|column| column.component_id() == component_id)?
            .get(row)
    }

    #[must_use]
    pub(crate) fn get_mut<T: Component>(
        &mut self,
        component_id: ComponentId,
        row: usize,
    ) -> Option<&mut T> {
        self.columns
            .iter_mut()
            .find(|column| column.component_id() == component_id)?
            .get_mut(row)
    }

    pub(crate) fn entity(&self, row: usize) -> Option<Entity> {
        self.entities.get(row).copied()
    }

    pub(crate) fn row_count(&self) -> usize {
        self.len()
    }

    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    #[must_use]
    pub(crate) fn is_full(&self) -> bool {
        self.len() == self.capacity
    }

    pub(crate) fn push_row(
        &mut self,
        entity: Entity,
        mut components: Vec<(ComponentId, Box<StoredComponent>)>,
    ) -> usize {
        assert!(!self.is_full());

        self.entities.push(entity);

        for column in &mut self.columns {
            let position = components
                .iter()
                .position(|(id, _)| *id == column.component_id())
                .expect("row is missing component for archetype column");

            let (_, value) = components.swap_remove(position);
            column.push(value)
        }

        assert!(
            components.is_empty(),
            "row contains components that do not belong to the archetype",
        );

        self.len() - 1
    }

    pub(crate) fn swap_remove_row(&mut self, row: usize) -> Option<Entity> {
        self.take_row(row).and_then(|row| row.moved_entity)
    }

    pub(crate) fn take_row(&mut self, row: usize) -> Option<RemovedChunkRow> {
        let last_row = self.len().checked_sub(1)?;
        let entity = self.entities.swap_remove(row);

        let components = self
            .columns
            .iter_mut()
            .map(|column| (column.component_id(), column.swap_remove(row)))
            .collect();

        let moved_entity = if row < last_row {
            Some(self.entities[row])
        } else {
            None
        };

        Some(RemovedChunkRow {
            entity,
            components,
            moved_entity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position(i32);

    #[derive(Debug, PartialEq)]
    struct Velocity(i32);

    #[test]
    fn swap_remove_row_returns_moved_entity() {
        let position_id = ComponentId::of::<Position>();
        let velocity_id = ComponentId::of::<Velocity>();

        let mut chunk = Chunk::new(&[position_id, velocity_id], 4);

        let first = Entity::new(0, 0);
        let second = Entity::new(1, 0);

        chunk.push_row(
            first,
            vec![
                (position_id, Box::new(Position(10))),
                (velocity_id, Box::new(Velocity(1))),
            ],
        );

        chunk.push_row(
            second,
            vec![
                (position_id, Box::new(Position(20))),
                (velocity_id, Box::new(Velocity(2))),
            ],
        );

        assert_eq!(chunk.swap_remove_row(0), Some(second));
        assert_eq!(chunk.len(), 1);
    }
}
