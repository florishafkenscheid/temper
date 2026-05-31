use crate::ecs::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Startup,
    FixedUpdate,
    Cleanup,
}

type BoxedSystem = Box<dyn FnMut(&mut World)>;

#[derive(Default)]
pub struct Schedule {
    startup: Vec<BoxedSystem>,
    fixed_update: Vec<BoxedSystem>,
    cleanup: Vec<BoxedSystem>,
}

impl Schedule {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system<S>(&mut self, stage: Stage, system: S)
    where
        S: FnMut(&mut World) + 'static,
    {
        self.systems_mut(stage).push(Box::new(system));
    }

    pub fn run_stage(&mut self, stage: Stage, world: &mut World) {
        for system in self.systems_mut(stage) {
            system(world);
        }
    }

    #[must_use]
    pub fn system_count(&self, stage: Stage) -> usize {
        self.systems(stage).len()
    }

    fn systems(&self, stage: Stage) -> &[BoxedSystem] {
        match stage {
            Stage::Startup => &self.startup,
            Stage::FixedUpdate => &self.fixed_update,
            Stage::Cleanup => &self.cleanup,
        }
    }

    fn systems_mut(&mut self, stage: Stage) -> &mut Vec<BoxedSystem> {
        match stage {
            Stage::Startup => &mut self.startup,
            Stage::FixedUpdate => &mut self.fixed_update,
            Stage::Cleanup => &mut self.cleanup,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, PartialEq)]
    struct ExecutionLog(Vec<&'static str>);

    #[test]
    fn systems_run_in_registration_order() {
        let mut world = World::new();
        world.insert_resource(ExecutionLog::default());

        let mut schedule = Schedule::new();

        schedule.add_system(Stage::FixedUpdate, |world| {
            world
                .get_resource_mut::<ExecutionLog>()
                .expect("ExecutionLog should exist")
                .0
                .push("first");
        });

        schedule.add_system(Stage::FixedUpdate, |world| {
            world
                .get_resource_mut::<ExecutionLog>()
                .expect("ExecutionLog should exist")
                .0
                .push("second");
        });

        schedule.run_stage(Stage::FixedUpdate, &mut world);

        assert_eq!(
            world.get_resource::<ExecutionLog>(),
            Some(&ExecutionLog(vec!["first", "second"]))
        );
    }

    #[test]
    fn running_stage_does_not_run_other_stages() {
        let mut world = World::new();
        world.insert_resource(ExecutionLog::default());

        let mut schedule = Schedule::new();

        schedule.add_system(Stage::Startup, |world| {
            world
                .get_resource_mut::<ExecutionLog>()
                .expect("ExecutionLog should exist")
                .0
                .push("startup");
        });

        schedule.add_system(Stage::Cleanup, |world| {
            world
                .get_resource_mut::<ExecutionLog>()
                .expect("ExecutionLog should exist")
                .0
                .push("cleanup");
        });

        schedule.run_stage(Stage::Startup, &mut world);

        assert_eq!(
            world.get_resource::<ExecutionLog>(),
            Some(&ExecutionLog(vec!["startup"]))
        );
    }

    #[test]
    fn fn_mut_system_keeps_state_between_runs() {
        let mut world = World::new();
        world.insert_resource(0_u64);

        let mut schedule = Schedule::new();
        let mut increment = 0_u64;

        schedule.add_system(Stage::FixedUpdate, move |world| {
            increment += 1;
            *world
                .get_resource_mut::<u64>()
                .expect("counter should exist") = increment;
        });

        schedule.run_stage(Stage::FixedUpdate, &mut world);
        schedule.run_stage(Stage::FixedUpdate, &mut world);

        assert_eq!(world.get_resource::<u64>(), Some(&2));
    }
}
