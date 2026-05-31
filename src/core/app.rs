use crate::{
    core::{
        plugin::Plugin,
        schedule::{Schedule, Stage},
    },
    ecs::{resource::Resource, world::World},
};

/// Temporary application shell.
///
/// This exists to establish the library boundary before the real app/plugin
/// runtime is implemented.
#[derive(Default)]
pub struct App {
    world: World,
    schedule: Schedule,
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
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    #[must_use]
    pub fn schedule_mut(&mut self) -> &mut Schedule {
        &mut self.schedule
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn add_system<S>(&mut self, stage: Stage, system: S) -> &mut Self
    where
        S: FnMut(&mut World) + 'static,
    {
        self.schedule.add_system(stage, system);
        self
    }

    pub fn run_stage(&mut self, stage: Stage) {
        self.schedule.run_stage(stage, &mut self.world);
    }

    pub fn run(&mut self) {
        self.run_stage(Stage::Startup);
        self.run_stage(Stage::FixedUpdate);
        self.run_stage(Stage::Cleanup);
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
        let mut app = App::new();
        app.insert_resource(Tick(10));

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(10)));
    }

    #[test]
    fn plugin_configures_app_world() {
        let mut app = App::new();
        app.add_plugin(TickPlugin { initial_tick: 42 });

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

        let mut app = App::new();
        app.add_plugin(TickPlugin { initial_tick: 10 })
            .add_plugin(IncrementTickPlugin);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(11)));
    }

    #[test]
    fn app_runs_registered_system_stage() {
        let mut app = App::new();

        app.insert_resource(Tick(10))
            .add_system(Stage::FixedUpdate, |world| {
                world
                    .get_resource_mut::<Tick>()
                    .expect("Tick should exist")
                    .0 += 1;
            });

        app.run_stage(Stage::FixedUpdate);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(11)));
    }

    #[test]
    fn plugin_registers_system() {
        struct IncrementTickPlugin;

        impl Plugin for IncrementTickPlugin {
            fn build(&self, app: &mut App) {
                app.add_system(Stage::FixedUpdate, |world| {
                    world
                        .get_resource_mut::<Tick>()
                        .expect("Tick should exist")
                        .0 += 1;
                });
            }
        }

        let mut app = App::new();

        app.insert_resource(Tick(10))
            .add_plugin(IncrementTickPlugin);

        app.run_stage(Stage::FixedUpdate);

        assert_eq!(app.world().get_resource::<Tick>(), Some(&Tick(11)));
    }
}
