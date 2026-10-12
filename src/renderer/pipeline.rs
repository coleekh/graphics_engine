use crate::{renderer::uniforms::InstanceData, scene::Vertex};

/// Build the PBR render pipeline from the embedded WGSL shader.
pub fn create_pbr_pipeline(
    device:     &wgpu::Device,
    format:      wgpu::TextureFormat,
    frame_bgl:  &wgpu::BindGroupLayout,
    material_bgl: &wgpu::BindGroupLayout,
    lights_capacity: u32,
) -> wgpu::RenderPipeline {
    // Embed shader at compile time so the binary is self-contained.
    let shader_src = include_str!("../../assets/shaders/opaque_lit.wgsl");
    let shader     = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label:  Some("pbr_shader"),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label:                Some("pbr_layout"),
        bind_group_layouts:   &[frame_bgl, material_bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label:  Some("pbr_pipeline"),
        layout: Some(&layout),

        vertex: wgpu::VertexState {
            module:      &shader,
            entry_point: "vs_main",
            buffers:     &[Vertex::layout(), InstanceData::layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },

        fragment: Some(wgpu::FragmentState {
            module:      &shader,
            entry_point: "fs_main",
            targets:     &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions {
                zero_initialize_workgroup_memory: true,
                constants: &std::collections::HashMap::from([
                    ("LIGHTS_CAPACITY".into(), lights_capacity.into()),
                ]),
            },
        }),

        primitive: wgpu::PrimitiveState {
            topology:           wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face:         wgpu::FrontFace::Ccw,
            cull_mode:          Some(wgpu::Face::Back),
            polygon_mode:       wgpu::PolygonMode::Fill,
            unclipped_depth:    false,
            conservative:       false,
        },

        depth_stencil: Some(wgpu::DepthStencilState {
            format:             wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare:      wgpu::CompareFunction::Less,
            stencil:            wgpu::StencilState::default(),
            bias:               wgpu::DepthBiasState::default(),
        }),

        multisample: wgpu::MultisampleState {
            count:                     1,
            mask:                      !0,
            alpha_to_coverage_enabled: false,
        },

        multiview: None,
    })
}

/// Build the depth-only pipeline that fills the shadow map.
///
/// There is deliberately no fragment stage: with one, the hardware has to
/// shade every fragment before it can test depth, which loses the early-Z
/// fast path and makes the pass scale with overdraw instead of with the
/// visible surface.
pub fn create_shadow_pipeline(
    device: &wgpu::Device,
    shadow_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_src = include_str!("../../assets/shaders/shadow_depth.wgsl");
    let shader     = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label:  Some("shadow_shader"),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label:                Some("shadow_layout"),
        bind_group_layouts:   &[shadow_bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label:  Some("shadow_pipeline"),
        layout: Some(&layout),

        vertex: wgpu::VertexState {
            module:      &shader,
            entry_point: "vs_main",
            buffers:     &[Vertex::layout(), InstanceData::shadow_layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },

        // No colour target at all: this pass only writes depth.
        fragment: None,

        primitive: wgpu::PrimitiveState {
            topology:           wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face:         wgpu::FrontFace::Ccw,
            cull_mode:          Some(wgpu::Face::Front),
            polygon_mode:       wgpu::PolygonMode::Fill,
            unclipped_depth:    false,
            conservative:       false,
        },

        depth_stencil: Some(wgpu::DepthStencilState {
            format:              wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare:       wgpu::CompareFunction::Less,
            stencil:             wgpu::StencilState::default(),
            // Bias is applied in the lit shader instead, where the surface's
            // slope to the light is known; doing it twice would over-bias.
            bias:                wgpu::DepthBiasState::default(),
        }),

        multisample: wgpu::MultisampleState {
            count:                     1,
            mask:                      !0,
            alpha_to_coverage_enabled: false,
        },

        multiview: None,
    })
}
