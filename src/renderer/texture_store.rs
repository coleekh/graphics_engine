use glam::{u8vec4, uvec4};
use wgpu::util::DeviceExt;

pub struct SamplerStore {
    pub linear_wrap: wgpu::Sampler,
}

pub struct TextureStore {
    pub white: wgpu::Texture,
    pub white_view: wgpu::TextureView,
    pub black: wgpu::Texture,
    pub black_view: wgpu::TextureView,
    pub unit_normal: wgpu::Texture,
    pub unit_normal_view: wgpu::TextureView,
}
 
 impl TextureStore {
     pub fn new(gpu: &crate::renderer::GpuContext) -> Self {
        fn pixel_texture(gpu: &crate::renderer::GpuContext, colour_data: &[u8], format: wgpu::TextureFormat) -> wgpu::Texture {
            assert!(colour_data.len() == 4, "Colour data must be 4 bytes (RGBA)");
            let pixel_dimensions = wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            };
            gpu.device.create_texture_with_data(&gpu.queue, &wgpu::TextureDescriptor {
                label: Some("white texture"),
                size: pixel_dimensions,
                dimension: wgpu::TextureDimension::D2,
                format,
                mip_level_count: 1,
                sample_count: 1,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
             }, wgpu::util::TextureDataOrder::default(), &colour_data)
        }

        let white = pixel_texture(gpu, &[255, 255, 255, 255], wgpu::TextureFormat::Rgba8Unorm);
        let black = pixel_texture(gpu, &[0, 0, 0, 255], wgpu::TextureFormat::Rgba8Unorm);
        let unit_normal = pixel_texture(gpu, bytemuck::bytes_of(&[0, 0, 127, 127i8]), wgpu::TextureFormat::Rgba8Snorm);
 
        let white_view = white.create_view(&wgpu::TextureViewDescriptor::default());
        let black_view = black.create_view(&wgpu::TextureViewDescriptor::default());
        let unit_normal_view = unit_normal.create_view(&wgpu::TextureViewDescriptor::default());
        
        Self { white, black, unit_normal, white_view, black_view, unit_normal_view }
    }
}

impl SamplerStore {
    pub fn new(gpu: &crate::renderer::GpuContext) -> Self {
        let linear_wrap = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear wrap sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self { linear_wrap }
    }
}