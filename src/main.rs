// #![feature(slice_as_array)]

use std::sync::Mutex;

use glfw::{Action, ClientApiHint, Key, WindowHint, fail_on_errors};
mod renderer;
use glm::Vector3;
use renderer::renderer::State;
mod model;
use model::{game_objects::Object, world::World};
mod physics;
mod utility;
use crate::physics::physics::PhysicsWorld;
use physics::*;
use rapier3d::math::Vector;
use rapier3d::prelude::*;
use std::sync::Arc;

pub struct AppState {
    pub phys_world: Mutex<PhysicsWorld>,
}

impl AppState {
    /// consume and guard the physics world
    pub fn new(world: PhysicsWorld) -> Self {
        AppState {
            phys_world: Mutex::new(world),
        }
    }
}

/// starts wgpu setup and loop
async fn run() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    let (mut window, events) = glfw
        .create_window(800, 600, "wgpu", glfw::WindowMode::Windowed)
        .unwrap();

    let mut state = State::new(&mut window).await;

    state.window.set_framebuffer_size_polling(true);
    state.window.set_key_polling(true);
    state.window.set_mouse_button_polling(true);
    state.window.set_pos_polling(true);
    state.window.set_cursor_mode(glfw::CursorMode::Hidden);

    state.load_assets("assets/companion_cube/companion_cube.obj");

    // Build world
    let mut world = World::new();
    world.tris.push(Object {
        position: glm::Vec3::new(0.0, 0.0, -1.0),
        angle: 0.0,
    });
    world.quads.push(Object {
        position: glm::Vec3::new(0.5, 0.0, -1.5),
        angle: 0.0,
    });
    state.build_ubos_for_objects(2);

    while !state.window.should_close() {
        glfw.poll_events();

        world.update(16.67, state.window);

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                //esc
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    state.window.set_should_close(true)
                }

                // fall back to world implementations
                glfw::WindowEvent::Key(key, _, Action::Press, _) => {
                    world.set_key(key, true);
                }
                glfw::WindowEvent::Key(key, _, Action::Release, _) => {
                    world.keys.insert(key, false);
                }

                /*world.tris.push(new_object);
                let true_count = world.tris.len(); // Or all objects if using a unified list

                if true_count > state.ubo.as_ref().unwrap().bind_groups.len() {
                    state.ubo = Some(UBOGroup::new(&state.device, true_count, &state.bind_group_layouts[&BindScope::UBO]));
                }

                // On each frame:
                state.ubo.as_mut().unwrap().upload(i as u64, &world.tris[i].calc_matrix(), &state.queue);
                 */
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

        match state.render(&world.quads, &world.tris, &world.camera) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                state.update_surface();
                state.resize(state.size);
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

/// starts the physics thread. This should only be running once
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
