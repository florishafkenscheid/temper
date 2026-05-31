use crate::ecs::{bundle::Bundle, entity::Entity, world::World};

type DeferredCommand = Box<dyn FnOnce(&mut World)>;

#[derive(Default)]
pub struct Commands {
    queued: Vec<DeferredCommand>,
}

impl Commands {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn<B>(&mut self, bundle: B)
    where
        B: Bundle + 'static,
    {
        self.queued.push(Box::new(move |world| {
            world.spawn(bundle);
        }));
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queued.push(Box::new(move |world| {
            world.despawn(entity);
        }));
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.queued.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queued.is_empty()
    }

    pub(crate) fn apply(self, world: &mut World) {
        for command in self.queued {
            command(world);
        }
    }
}
