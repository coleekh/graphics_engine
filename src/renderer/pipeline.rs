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
    let shader_src = include_str!("../../assets/shaders/pbr.wgsl");
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
