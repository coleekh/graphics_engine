mod renderer;
mod scene;
mod utils;
mod input;

use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey}, window,
};

use renderer::Engine;
use scene::{Scene, camera::Camera, Light, LightKind, MeshBuilder};
use glam::{DVec2, Quat, Vec2, Vec3, dvec2};

fn main() {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting graphics engine...");

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let mut app = App { 
        window: None,
        last_frame: std::time::Instant::now(), 
        total_time: 0.0,
        cursor_position: Vec2::NAN,
        cursor_motion: DVec2::ZERO,
    };

    event_loop.run_app(&mut app).expect("Event loop error");
}

struct App {
    window: Option<(winit::window::Window, Engine, Scene)>,
    last_frame: std::time::Instant,
    total_time: f32,
    cursor_position: Vec2,
    cursor_motion: DVec2,
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("Application resumed.");

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        
        let window = event_loop.create_window(
            winit::window::WindowAttributes::default()
            .with_title("wgpu Graphics Engine")
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
            .with_resizable(true)
        ).expect("Failed to create window on resume");

        // Initialize engine (blocks until GPU adapter/device are ready)
        let mut engine = pollster::block_on(Engine::new(&window));

        // Build the initial scene
        let scene = build_demo_scene(&mut engine);

        // Request an initial redraw to kick off the rendering loop
        window.request_redraw();
        
        self.window = Some((window, engine, scene));
    }
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        event_loop.set_control_flow(ControlFlow::Poll);

        if let Some((window, engine, scene)) = &mut self.window {
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
                        physical_key: PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                } => {
                    handle_key(&key, scene, event_loop, window);
                }

                WindowEvent::CursorMoved { device_id, position } => {
                    // let new_cursor_position = Vec2::new(position.x as f32, position.y as f32);
                    // if self.cursor_position.is_nan() {
                    //     self.cursor_position = new_cursor_position; // Initialize on first event
                    // }
                    // self.cursor_motion = new_cursor_position - self.cursor_position; // This will be (0.0, 0.0) for the first event
                    // self.cursor_position = new_cursor_position;
                }

                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = now.duration_since(self.last_frame).as_secs_f32();
                    self.last_frame = now;
                    self.total_time += dt;
                    
                    scene.camera.global_yaw(-self.cursor_motion.x as f32 * 0.002);
                    scene.camera.pitch(-self.cursor_motion.y as f32 * 0.002);
                
                    // Animate scene objects
                    scene.update(self.total_time, dt);

                    // scene.camera.orientation = Quat::look_at_rh(scene.camera.position, Vec3::ZERO, scene.camera.up);
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
                    self.cursor_motion = DVec2::ZERO; // Reset cursor motion after processing
                }

                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some((window, engine, scene)) = &mut self.window {
            match event {
                winit::event::DeviceEvent::MouseMotion { delta: (delta_x, delta_y) } => {
                    self.cursor_motion = dvec2(delta_x, delta_y);
                }
                _ => {}
            }   
        }   
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
        color: Vec3::new(0.3, 0.6, 1.0),
        intensity: 2.0,
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
    let cube_mesh = MeshBuilder::cube(1.0);
    let cube_handle = engine.upload_mesh(&cube_mesh);
    scene.objects.push(scene::SceneObject {
        mesh: cube_handle,
        position: Vec3::new(0.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
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
    fn update(&mut self, time: f32, dt: f32) {
        // Rotate cube (index 1)
        if let Some(obj) = self.objects.get_mut(1) {
            obj.rotation = Quat::from_rotation_y(time * 0.8)
                * Quat::from_rotation_x(time * 0.3);
        }

        // Bob sphere (index 2)
        if let Some(obj) = self.objects.get_mut(2) {
            obj.position.y = 0.5 + (time * 1.2).sin() * 0.4;
            obj.rotation = Quat::from_rotation_y(time * -0.5);
        }

        // Spin torus (index 3)
        if let Some(obj) = self.objects.get_mut(3) {
            obj.rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(time * 1.1);
        }

        // Orbit the point light and its orb proxy (index 4)
        let orbit_r = 3.0f32;
        let orbit_speed = 0.7;
        let lx = orbit_r * (time * orbit_speed).cos();
        let lz = orbit_r * (time * orbit_speed).sin();
        if let Some(light) = self.lights.get_mut(1) {
            light.position = Vec3::new(lx, 2.0, lz);
        }
        if let Some(obj) = self.objects.get_mut(4) {
            obj.position = Vec3::new(lx, 2.0, lz);
        }
    }
}

fn handle_key(
    key: &KeyCode,
    scene: &mut Scene,
    event_loop: &winit::event_loop::ActiveEventLoop,
    window: &winit::window::Window,
) {
    let cam = &mut scene.camera;
    let speed = 0.5;

    match key {
        KeyCode::KeyW | KeyCode::ArrowUp    => { cam.translate(Vec3::NEG_Z * speed); }
        KeyCode::KeyS | KeyCode::ArrowDown  => { cam.translate(Vec3::NEG_Z * -speed); }
        KeyCode::KeyA | KeyCode::ArrowLeft  => { cam.translate(Vec3::X * -speed); }
        KeyCode::KeyD | KeyCode::ArrowRight => { cam.translate(Vec3::X * speed); }
        KeyCode::KeyQ                       => { cam.translate(Vec3::Y * -speed); }
        KeyCode::KeyE                       => { cam.translate(Vec3::Y * speed); }
        KeyCode::Escape                     => { event_loop.exit(); }
        KeyCode::Tab                        => { 
            window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap(); 
            window.set_cursor_visible(false);
        }
        _ => {}
    }
}
