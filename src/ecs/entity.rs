#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    index: u32,
    generation: u32,
}

impl Entity {
    #[must_use]
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    #[must_use]
    pub fn index(&self) -> u32 {
        self.index
    }

    #[must_use]
    pub fn generation(&self) -> u32 {
        self.generation
    }
}

#[derive(Debug)]
struct EntitySlot {
    generation: u32,
    alive: bool,
}

#[derive(Debug, Default)]
pub struct EntityAllocator {
    slots: Vec<EntitySlot>,
    free: Vec<u32>,
}

impl EntityAllocator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self) -> Entity {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.alive = true;
            Entity::new(index, slot.generation)
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(EntitySlot {
                generation: 0,
                alive: true,
            });
            Entity::new(index, 0)
        }
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        let Some(slot) = self.slots.get_mut(entity.index as usize) else {
            return false;
        };

        if !slot.alive || slot.generation != entity.generation {
            return false;
        };

        slot.alive = false;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(entity.index);
        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.slots
            .get(entity.index as usize)
            .is_some_and(|slot| slot.alive && slot.generation == entity.generation)
    }

    pub fn alive_count(&self) -> usize {
        self.slots.len() - self.free.len()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn spawned_entity_is_alive() {
        let mut entities = EntityAllocator::new();
        let entity = entities.spawn();
        assert!(entities.is_alive(entity));
    }

    #[test]
    fn despawn_entity_is_not_alive() {
        let mut entities = EntityAllocator::new();
        let entity = entities.spawn();
        assert!(entities.despawn(entity));
        assert!(!entities.is_alive(entity));
    }

    #[test]
    fn despawn_rejects_stale_entity() {
        let mut entities = EntityAllocator::new();
        let entity = entities.spawn();
        entities.despawn(entity);
        assert!(!entities.despawn(entity));
    }

    #[test]
    fn allocator_reuses_index_with_new_generation() {
        let mut entities = EntityAllocator::new();
        let first = entities.spawn();
        assert!(entities.despawn(first));
        let second = entities.spawn();
        assert_ne!(first.generation(), second.generation());
        assert_eq!(first.index(), second.index());
    }

    #[test]
    fn despawn_rejects_unknown_entity() {
        let mut entities = EntityAllocator::new();
        assert_eq!(
            entities.despawn(Entity {
                index: 0,
                generation: 0
            }),
            false
        );
    }

    #[test]
    fn reused_slot_invalidates_old_entity() {
        let mut entities = EntityAllocator::new();

        let first = entities.spawn();
        assert!(entities.despawn(first));

        let second = entities.spawn();

        assert_eq!(first.index(), second.index());
        assert_ne!(first.generation(), second.generation());
        assert!(!entities.is_alive(first));
        assert!(entities.is_alive(second));
    }
}
