use crate::{
    core::plugin::Plugin,
    ecs::{resource::Resource, world::World},
};

/// Temporary application shell.
///
/// This exists to establish the library boundary before the real app/plugin
/// runtime is implemented.
#[derive(Default)]
pub struct App {
    world: World,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

    #[must_use]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    #[must_use]
    pub fn insert_resource<T: Resource>(mut self, resource: T) -> Self {
        self.world.insert_resource(resource);
        self
    }

    #[must_use]
    pub fn add_plugin<P: Plugin>(mut self, plugin: P) -> Self {
        plugin.build(&mut self);
        self
    }

    pub fn run(self) {
        // Real runtime loop comes later.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Tick(u64);

    struct TickPlugin {
        initial_tick: u64,
    }

    impl Plugin for TickPlugin {
        fn build(&self, app: &mut App) {
            app.world_mut().insert_resource(Tick(self.initial_tick));
        }
    }

    #[test]
    fn app_owns_world() {
        let mut app = App::new();

        let entity = app.world_mut().spawn((Tick(1),));

        assert!(app.world().is_alive(entity));
    }

    #[test]
    fn insert_resource_configures_world() {
        let app = App::new().insert_resource(Tick(10));

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(10)));
    }

    #[test]
    fn plugin_configures_app_world() {
        let app = App::new().add_plugin(TickPlugin { initial_tick: 42 });

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(42)));
    }

    #[test]
    fn plugins_apply_in_registration_order() {
        struct IncrementTickPlugin;

        impl Plugin for IncrementTickPlugin {
            fn build(&self, app: &mut App) {
                app.world_mut()
                    .get_resource_mut::<Tick>()
                    .expect("Tick should exist")
                    .0 += 1;
            }
        }

        let app = App::new()
            .add_plugin(TickPlugin { initial_tick: 10 })
            .add_plugin(IncrementTickPlugin);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(11)));
    }
}
