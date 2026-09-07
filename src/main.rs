mod renderer;
mod scene;
mod utils;
mod input;
mod time;

use std::sync::Arc;

use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use renderer::Engine;
use scene::{Scene, Light, LightKind, mesh_builder::MeshBuilder};
use glam::{Quat, Vec3, vec3};

fn main() {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting graphics engine...");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    
    let mut app = App { 
        engine: None,
        clock: time::Clock::now(),
        input: input::InputBinder::default(),
        mouse_captured: false,
    };

    event_loop.run_app(&mut app).expect("Event loop error");
}

struct App<'a> {
    engine: Option<(Arc<winit::window::Window>, Engine<'a>, Scene)>,
    clock: time::Clock,
    input: input::InputBinder,
    mouse_captured: bool,
}

impl<'a> winit::application::ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("Application resumed.");

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        
        let window = event_loop.create_window(
            winit::window::WindowAttributes::default()
            .with_title("wgpu Graphics Engine")
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
            .with_resizable(true)
        ).expect("Failed to create window on resume");
        let window = Arc::new(window);

        // Initialize engine (blocks until GPU adapter/device are ready)
        let mut engine = pollster::block_on(Engine::new(Arc::clone(&window)));

        // Build the initial scene
        let scene = build_demo_scene(&mut engine);

        // Request an initial redraw to kick off the rendering loop
        window.request_redraw();
        
        self.engine = Some((window, engine, scene));
    }
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        event_loop.set_control_flow(ControlFlow::Poll);

        if let Some((window, engine, scene)) = &mut self.engine {
            if window.id() != window_id {
                return; // Ignore events for other windows (if any)
            }

            match event {
                WindowEvent::CloseRequested => {
                    log::info!("Window close requested.");
                    event_loop.exit();
                }

                WindowEvent::Resized(size) => {
                    engine.resize(size.width, size.height);
                    scene.camera.aspect = size.width as f32 / size.height as f32;
                    log::info!("Resized to {}x{}", size.width, size.height);
                }

                WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                } => {
                    if key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) { 
                        event_loop.exit(); 
                    }
                }

                WindowEvent::CursorMoved { device_id, position } => {}

                WindowEvent::RedrawRequested => {
                    if self.input[input::ActionBinding::MouseCapture] == input::KeyState::JustPressed {
                        if self.mouse_captured {
                            window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap(); 
                            window.set_cursor_visible(true);
                            self.mouse_captured = false;
                        } else {
                            window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap(); 
                            window.set_cursor_visible(false);
                            self.mouse_captured = true;
                        } 
                    }
                    
                    scene.update(self.clock, &self.input);

                    match engine.render(&scene) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let size = window.inner_size();
                            engine.resize(size.width, size.height);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("Out of GPU memory!");
                            event_loop.exit();
                        }
                        Err(e) => log::warn!("Surface error: {:?}", e),
                    }

                    window.request_redraw();
                    self.end_frame();           
                }

                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.input.process_device_event(device_id, event);
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("Closing event loop.")
    }
}

impl<'a> App<'a> {
    /// Start next frame.
    fn end_frame(&mut self) {
        self.clock = self.clock.next_frame();
        self.input.end_frame();
    }
}

fn build_demo_scene(engine: &mut Engine) -> Scene {
    let mut scene = Scene::new();

    // Camera positioned to view the scene
    scene.camera = scene::camera::Camera::new(
        Vec3::new(0.0, 0.0, 10.0),
        1280.0 / 720.0,
    );

    // Lights
    scene.lights.push(Light {
        kind: LightKind::Directional,
        position: Vec3::new(5.0, 10.0, 5.0),
        direction: Vec3::new(-1.0, -2.0, -1.0).normalize(),
        color: Vec3::new(1.0, 0.95, 0.85),
        intensity: 1.5,
        range: 0.0,
    });

    scene.lights.push(Light {
        kind: LightKind::Point,
        position: Vec3::new(-3.0, 2.0, 0.0),
        direction: Vec3::ZERO,
        color: Vec3::new(1.0, 1.0, 1.0),
        intensity: 10.0,
        range: 10.0,
    });

    // Ground plane (flat quad)
    let ground_mesh = MeshBuilder::plane(10.0, 10.0, 20, 20);
    let ground_handle = engine.upload_mesh(&ground_mesh);
    scene.objects.push(scene::SceneObject {
        mesh: ground_handle,
        position: Vec3::new(0.0, -1.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        material: scene::Material {
            base_color: Vec3::new(0.3, 0.32, 0.35),
            metallic: 0.0,
            roughness: 0.9,
            emissive: Vec3::ZERO,
        },
    });

    // Center cube
    let cube_mesh = MeshBuilder::unit_cube();
    let cube_handle = engine.upload_mesh(&cube_mesh);
    scene.objects.push(scene::SceneObject {
        mesh: cube_handle,
        position: Vec3::new(0.0, 0.0, 0.0),
        rotation: Quat::from_axis_angle(vec3(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_3),
        scale: Vec3::ONE,
        material: scene::Material {
            base_color: Vec3::new(0.8, 0.2, 0.2),
            metallic: 0.1,
            roughness: 0.4,
            emissive: Vec3::ZERO,
        },
    });

    // Sphere
    let sphere_mesh = MeshBuilder::sphere(0.8, 32, 32);
    let sphere_handle = engine.upload_mesh(&sphere_mesh);
    scene.objects.push(scene::SceneObject {
        mesh: sphere_handle,
        position: Vec3::new(3.0, 0.5, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        material: scene::Material {
            base_color: Vec3::new(0.2, 0.5, 0.9),
            metallic: 0.8,
            roughness: 0.2,
            emissive: Vec3::ZERO,
        },
    });

    // Torus
    let torus_mesh = MeshBuilder::torus(1.0, 0.35, 32, 16);
    let torus_handle = engine.upload_mesh(&torus_mesh);
    scene.objects.push(scene::SceneObject {
        mesh: torus_handle,
        position: Vec3::new(-3.0, 0.5, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        material: scene::Material {
            base_color: Vec3::new(0.9, 0.7, 0.1),
            metallic: 0.6,
            roughness: 0.3,
            emissive: Vec3::ZERO,
        },
    });

    // Small emissive orb (acts as point-light visualization)
    let orb_mesh = MeshBuilder::sphere(0.15, 12, 12);
    let orb_handle = engine.upload_mesh(&orb_mesh);
    scene.objects.push(scene::SceneObject {
        mesh: orb_handle,
        position: Vec3::new(-3.0, 2.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        material: scene::Material {
            base_color: Vec3::ZERO,
            metallic: 0.0,
            roughness: 1.0,
            emissive: Vec3::new(0.3, 0.6, 1.0) * 3.0,
        },
    });

    log::info!("Scene built with {} objects.", scene.objects.len());
    scene
}

impl Scene {
    fn update(&mut self, clock: time::Clock, input: &input::InputBinder) {
        use input::ActionBinding::*;

        self.camera.global_yaw(-input.mouse_motion().x as f32);
        self.camera.pitch(-input.mouse_motion().y as f32);
        let speed = 6.0;
        let strafe = match (input[MoveRight].is_pressed(), input[MoveLeft].is_pressed()) {
            (true, false)                 => 1.0,
            (false, true)                => -1.0,
            (true, true) | (false, false) => 0.0,
        };
        let fly = match (input[MoveUp].is_pressed(), input[MoveDown].is_pressed()) {
            (true, false)                 => 1.0,
            (false, true)                => -1.0,
            (true, true) | (false, false) => 0.0,
        };
        let run = match (input[MoveForward].is_pressed(), input[MoveBack].is_pressed()) {
            (true, false)                => -1.0,
            (false, true)                 => 1.0,
            (true, true) | (false, false) => 0.0,
        };
        self.camera.translate(clock.delta_time() as f32 * vec3(strafe, fly, run) * speed);

        // Rotate cube (index 1)
        // if let Some(obj) = self.objects.get_mut(1) {
        //     obj.rotation = Quat::from_rotation_y(clock.total_time() as f32 * 0.8)
        //     // obj.rotation = Quat::from_rotation_y(clock.total_time() as f32 * 0.8)
        //     //     * Quat::from_rotation_x(clock.total_time() as f32 * 0.3);
        // }

        // // Bob sphere (index 2)
        // if let Some(obj) = self.objects.get_mut(2) {
        //     obj.position.y = 0.5 + (clock.total_time() as f32 * 1.2).sin() * 0.4;
        //     obj.rotation = Quat::from_rotation_y(clock.total_time() as f32 * -0.5);
        // }

        // // Spin torus (index 3)
        // if let Some(obj) = self.objects.get_mut(3) {
        //     obj.rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
        //         * Quat::from_rotation_z(clock.total_time() as f32 * 1.1);
        // }

        // Orbit the point light and its orb proxy (index 4)
        let orbit_r = 3.0f32;
        let orbit_speed = 0.7;
        let lx = orbit_r * (clock.total_time() as f32 * orbit_speed).cos();
        let lz = orbit_r * (clock.total_time() as f32 * orbit_speed).sin();
        if let Some(light) = self.lights.get_mut(1) {
            light.position = Vec3::new(lx, 2.0, lz);
        }
        if let Some(obj) = self.objects.get_mut(4) {
            obj.position = Vec3::new(lx, 2.0, lz);
        }
    }
}

// fn handle_key(
//     key: &input::ActionBinding,
//     scene: &mut Scene,
//     window: &winit::window::Window,
// ) {
//     let cam = &mut scene.camera;
//     let speed = 0.5;

//     match key {
//         input::ActionBinding::MoveForward => { cam.translate(Vec3::NEG_Z * speed); }
//         input::ActionBinding::MoveBack => { cam.translate(Vec3::NEG_Z * -speed); }
//         input::ActionBinding::MoveLeft => { cam.translate(Vec3::X * -speed); }
//         input::ActionBinding::MoveRight => { cam.translate(Vec3::X * speed); }
//         input::ActionBinding::MoveDown => { cam.translate(Vec3::Y * -speed); }
//         input::ActionBinding::MoveUp => { cam.translate(Vec3::Y * speed); }
//         input::ActionBinding::MouseCapture => { 
//             window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap(); 
//             window.set_cursor_visible(false);
//         }
//         _ => {}
//     }
// }
