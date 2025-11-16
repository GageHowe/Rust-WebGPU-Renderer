// #![feature(slice_as_array)]

use std::sync::Mutex;

use glfw::{Action, ClientApiHint, Key, WindowHint, fail_on_errors};
mod renderer;
use renderer::backend::definitions::{Camera, InstanceData};
use renderer::renderer::RendererState;
// mod model;
// use model::game_objects::*;
mod physics;
// mod utility;
use crate::physics::physics::PhysicsWorld;
use glam::*;
use physics::*;
use rapier3d::math::Vector;
use rapier3d::prelude::*;
use std::sync::Arc;

pub struct AppState {
    pub phys_world: Mutex<PhysicsWorld>,
}

impl AppState {
    /// consume and wrap the physics world reference
    pub fn new(world: PhysicsWorld) -> Self {
        AppState {
            phys_world: Mutex::new(world),
        }
    }
}

fn update_camera(camera: &mut Camera, dt: f32, window: &mut glfw::Window) {
    let mouse_pos = window.get_cursor_pos();
    window.set_cursor_pos(400.0, 300.0);
    let dx = (-40.0 * (mouse_pos.0 - 400.0) / 400.0) as f32;
    let dy = (-40.0 * (mouse_pos.1 - 300.0) / 300.0) as f32;
    camera.spin(dx, dy);
}

async fn run() {
    let mut object_instances: Vec<InstanceData> = vec![];
    let mut camera = Camera::new();

    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    let (mut window, events) = glfw
        .create_window(800, 600, "wgpu", glfw::WindowMode::Windowed)
        .unwrap();

    let mut state = RendererState::new(&mut window).await;

    state.window.set_framebuffer_size_polling(true);
    state.window.set_key_polling(true);
    state.window.set_mouse_button_polling(true);
    state.window.set_pos_polling(true);
    state.window.set_cursor_mode(glfw::CursorMode::Hidden);

    state.load_assets("assets/companion_cube/companion_cube.obj");
    state.load_assets("assets/companion_cube/companion_cube.obj");

    // without this the objects fail to render???
    // yeah dumbass, it's an instance of the object
    object_instances.push(InstanceData {
        position: Vec3::new(0.0, 0.0, 20.0),
        rotation: glam::quat(0.0, 0.0, 0.0, 0.0),
    });

    object_instances.push(InstanceData {
        position: Vec3::new(0.0, 0.0, 0.0),
        // angle: 0.0,
        rotation: glam::quat(0.0, 0.0, 0.0, 0.0),
    });

    // build_ubos_for_objects(2);
    state.build_ubos_for_objects(object_instances.len());
    // state.update_instance_buffer(&object_instances);

    while !state.window.should_close() {
        glfw.poll_events();

        update_camera(&mut camera, 16.67, state.window);

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                //esc
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    state.window.set_should_close(true)
                }

                // window moved
                glfw::WindowEvent::Pos(..) => {
                    state.update_surface();
                    state.resize(state.size);
                }

                // window resized
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    state.update_surface();
                    state.resize((width, height));
                }
                _ => {}
            }
        }

        match state.render(&object_instances, &camera) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                state.update_surface();
                state.resize(state.size);
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

fn physics_thread(appstate: AppState) {
    println!("Physics thread starting");

    let mut physics = appstate.phys_world.lock().unwrap();

    let ground = ColliderBuilder::cuboid(100.0, 0.1, 100.0).build();

    {
        // does this work?
        physics.collider_set.insert(ground);
    }
    let ball = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, 10.0, 0.0])
        .build();
    let ball_collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
    let ball_handle = physics.rigid_body_set.insert(ball);

    let PhysicsWorld {
        // weird borrow checker worship
        rigid_body_set,
        collider_set,
        ..
    } = &mut *physics;
    collider_set.insert_with_parent(ball_collider, ball_handle, rigid_body_set);

    for _ in 0..200 {
        physics.step();

        let ball_body = &physics.rigid_body_set[ball_handle];
        println!("Ball altitude: {}", ball_body.translation().y);
    }
}

fn main() {
    // let physics = PhysicsWorld::new(Vector::new(0.0, -9.81, 0.0));
    // let global_app_state = AppState::new(physics);

    // std::thread::spawn(move || physics_thread(global_app_state));
    pollster::block_on(run());
}
