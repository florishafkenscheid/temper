use crate::ecs::{bundle::Bundle, component::Component, entity::Entity, world::World};

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

    pub fn insert_component<T>(&mut self, entity: Entity, component: T)
    where
        T: Component,
    {
        self.queued.push(Box::new(move |world| {
            world.insert_component(entity, component);
        }));
    }

    pub fn remove_component<T>(&mut self, entity: Entity)
    where
        T: Component,
    {
        self.queued.push(Box::new(move |world| {
            world.remove_component::<T>(entity);
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
