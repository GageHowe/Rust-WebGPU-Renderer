// #![feature(slice_as_array)]

// SHIT
// RAPIER GIVES US AABB BOXES
// SO WE CAN DO OCCLUSION QUERIES RIGHT???

use std::sync::Mutex;

use glfw::{Action, ClientApiHint, Key, WindowHint, fail_on_errors};
mod renderer;
use renderer::backend::definitions::{Camera, InstanceData};
use renderer::renderer::RendererState;
mod physics;
use crate::physics::physics::PhysicsWorld;
use glam::*;
use physics::*;
use rand::Rng;
use rapier3d::math::Vector;
use rapier3d::prelude::*;
use std::sync::Arc;
mod game_object;
use game_object::*;

// TODO: implement occlusion and frustum culling

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
    let speed = 0.5 * dt;

    let mouse_pos = window.get_cursor_pos();
    window.set_cursor_pos(400.0, 300.0);
    let dx = (-40.0 * (mouse_pos.0 - 400.0) / 400.0) as f32;
    let dy = (-40.0 * (mouse_pos.1 - 300.0) / 300.0) as f32;
    camera.look(dx, dy);

    if window.get_key(Key::W) == Action::Press {
        camera.position += camera.forwards * speed;
    }
    if window.get_key(Key::S) == Action::Press {
        camera.position -= camera.forwards * speed;
    }
    if window.get_key(Key::A) == Action::Press {
        camera.position -= camera.right * speed;
    }
    if window.get_key(Key::D) == Action::Press {
        camera.position += camera.right * speed;
    }
    if window.get_key(Key::Space) == Action::Press {
        camera.position += camera.up * speed;
    }
    if window.get_key(Key::LeftShift) == Action::Press {
        camera.position -= camera.up * speed;
    }
}

async fn run() {
    // let mut object_instances: Vec<InstanceData> = vec![];
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

    // spawn a bunch of instances
    let mut rng = rand::rng();
    let spacing = 50.0;
    for y in 0..100 {
        for x in 0..100 {
            // grid position
            let pos = glam::vec3(x as f32 * spacing, 0.0, y as f32 * spacing);

            // random orientation (random unit quaternion)
            let rand_axis = glam::Vec3::new(
                rng.random_range(-1.0..1.0),
                rng.random_range(-1.0..1.0),
                rng.random_range(-1.0..1.0),
            )
            .normalize_or_zero();

            let rand_angle = rng.random_range(0.0..std::f32::consts::TAU);

            let rot = glam::Quat::from_axis_angle(rand_axis, rand_angle);

            state
                .object_instances
                .push(InstanceData::from_pos_rot(pos, rot, 1.0));
        }
    }

    while !state.window.should_close() {
        glfw.poll_events();

        update_camera(&mut camera, 16.67, state.window);

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                // esc
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    state.window.set_should_close(true)
                }

                // // window moved
                // glfw::WindowEvent::Pos(..) => {
                //     state.update_surface();
                //     state.resize(state.size);
                // }

                // window resized
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // state.update_surface();
                    state.resize((width, height));
                }
                _ => {}
            }
        }

        let x = state.object_instances.clone(); // TODO: find a more clean way of doing this
        match state.render(&x, &camera) {
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
