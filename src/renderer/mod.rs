mod gpu;
mod pipeline;
mod mesh_store;
mod uniforms;

pub use gpu::GpuContext;
pub use mesh_store::MeshStore;

use winit::window::Window;
use wgpu::util::DeviceExt;

use crate::scene::{Scene, Mesh, MeshHandle};
use uniforms::{CameraUniform, LightUniform, ObjectUniform, MAX_LIGHTS};

// ─── Engine ──────────────────────────────────────────────────────────────────

/// Top-level graphics engine. Owns the GPU context, pipelines, and mesh store.
pub struct Engine<'a> {
    pub gpu:        GpuContext<'a>,
    pub mesh_store: MeshStore,

    // Render pipeline
    pbr_pipeline:   wgpu::RenderPipeline,

    // Bind group layout for camera + lights (set 0)
    frame_bgl:      wgpu::BindGroupLayout,
    // Bind group layout for per-object data (set 1)
    object_bgl:     wgpu::BindGroupLayout,

    // Per-frame GPU uniform buffers
    camera_buf:     wgpu::Buffer,
    lights_buf:     wgpu::Buffer,
    frame_bg:       wgpu::BindGroup,

    // Depth texture
    depth_texture:  wgpu::Texture,
    depth_view:     wgpu::TextureView,
}

impl<'a> Engine<'a> {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let gpu = GpuContext::new(window).await;

        // ── Bind Group Layouts ────────────────────────────────────────────

        let frame_bgl = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame_bgl"),
            entries: &[
                // camera uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // lights uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let object_bgl = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("object_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // ── Uniform Buffers ────────────────────────────────────────────────

        let camera_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera_uniform"),
            size:  std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lights_size = std::mem::size_of::<LightUniform>() * MAX_LIGHTS + 16; // +count padding
        let lights_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights_uniform"),
            size:  lights_size as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let frame_bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame_bg"),
            layout: &frame_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: lights_buf.as_entire_binding(),
                },
            ],
        });

        // ── Depth Texture ─────────────────────────────────────────────────

        let (depth_texture, depth_view) = Self::create_depth_texture(
            &gpu.device,
            gpu.surface_config.width,
            gpu.surface_config.height,
        );

        // ── Pipeline ──────────────────────────────────────────────────────

        let pbr_pipeline = pipeline::create_pbr_pipeline(
            &gpu.device,
            gpu.surface_config.format,
            &frame_bgl,
            &object_bgl,
        );

        Engine {
            mesh_store: MeshStore::new(),
            gpu,
            pbr_pipeline,
            frame_bgl,
            object_bgl,
            camera_buf,
            lights_buf,
            frame_bg,
            depth_texture,
            depth_view,
        }
    }

    // ── Public API ────────────────────────────────────────────────────────

    /// Upload a CPU mesh to the GPU and return a handle.
    pub fn upload_mesh(&mut self, mesh: &Mesh) -> MeshHandle {
        self.mesh_store.upload(&self.gpu.device, mesh)
    }

    /// Resize surface and recreate depth texture.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 { return; }
        self.gpu.surface_config.width  = width;
        self.gpu.surface_config.height = height;
        self.gpu.surface.configure(&self.gpu.device, &self.gpu.surface_config);
        let (dt, dv) = Self::create_depth_texture(&self.gpu.device, width, height);
        self.depth_texture = dt;
        self.depth_view    = dv;
    }

    /// Render a scene.
    pub fn render(&mut self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let output  = self.gpu.surface.get_current_texture()?;
        let view    = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // ── Upload camera uniform ─────────────────────────────────────────
        let cam_uni = CameraUniform::from_camera(&scene.camera);
        self.gpu.queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&cam_uni));

        // ── Upload lights uniform ─────────────────────────────────────────
        let (lights_data, count) = LightUniform::from_scene_lights(&scene.lights);
        let count = count.min(MAX_LIGHTS as usize) as u32; // clamp to max;
        let count_bytes = bytemuck::bytes_of(&count);
        // write count first (aligned)
        self.gpu.queue.write_buffer(&self.lights_buf, 0, count_bytes);
        // pad to 16 bytes for alignment
        self.gpu.queue.write_buffer(&self.lights_buf, 16,
            bytemuck::cast_slice(&lights_data));

        // ── Encoder ──────────────────────────────────────────────────────
        let mut encoder = self.gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("render_encoder") }
        );

        let objects: Vec<_> = scene.objects.iter().map(|obj| {
            // Create per-object uniform + bind group
            let obj_uni = ObjectUniform::from_object(obj);
            let obj_buf = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label:    Some("object_uniform"),
                contents: bytemuck::bytes_of(&obj_uni),
                usage:    wgpu::BufferUsages::UNIFORM,
            });
            let obj_bg = self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label:  Some("object_bg"),
                layout: &self.object_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding:  0,
                    resource: obj_buf.as_entire_binding(),
                }],
            });
            (obj_bg, obj.mesh)
        }).collect();


        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05, g: 0.05, b: 0.08, a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.pbr_pipeline);
            rpass.set_bind_group(0, &self.frame_bg, &[]);

            for (obj_bg, obj_mesh) in objects.iter() {
                rpass.set_bind_group(1, obj_bg, &[]);

                if let Some(gpu_mesh) = self.mesh_store.get(*obj_mesh) {
                    rpass.set_vertex_buffer(0, gpu_mesh.vertex_buf.slice(..));
                    rpass.set_index_buffer(gpu_mesh.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
                }
            }
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    // ── Private helpers ───────────────────────────────────────────────────

    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}
