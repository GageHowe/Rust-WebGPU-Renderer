use nalgebra::Vector3;
use rapier3d::prelude::*;
// use std::sync::Mutex;

pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub gravity: Vector3<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub physics_hooks: (),
    pub event_handler: (),
}

impl PhysicsWorld {
    pub fn new(gravity: Vector3<f32>) -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity,
            integration_parameters: IntegrationParameters {
                dt: 1.0 / 60.0,
                min_ccd_dt: 1.0 / 60.0 / 100.0,
                contact_natural_frequency: 30.0,
                contact_damping_ratio: 5.0,
                joint_natural_frequency: 1.0e6,
                joint_damping_ratio: 1.0,
                warmstart_coefficient: 1.0,
                num_internal_pgs_iterations: 1,
                num_internal_stabilization_iterations: 1,
                num_solver_iterations: 4,
                // TODO: what is the optimal value for min_island_size?
                // It should not be too big so that we don't end up with
                // huge islands that don't fit in cache.
                // However we don't want it to be too small and end up with
                // tons of islands, reducing SIMD parallelism opportunities.
                min_island_size: 128,
                normalized_allowed_linear_error: 0.001,
                normalized_max_corrective_velocity: 10.0,
                normalized_prediction_distance: 0.002,
                max_ccd_substeps: 1,
                length_unit: 1.0,
                friction_model: FrictionModel::default(),
            },
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        );
    }
}
