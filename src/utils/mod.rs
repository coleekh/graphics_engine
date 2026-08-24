/// Utility helpers for the graphics engine.

// ─── Texture loader ──────────────────────────────────────────────────────────

/// Load an RGBA image from disk and upload it to the GPU.
/// Returns (texture, view, sampler).
pub fn load_texture(
    device:   &wgpu::Device,
    queue:    &wgpu::Queue,
    path:     &std::path::Path,
    label:    &str,
) -> Result<(wgpu::Texture, wgpu::TextureView, wgpu::Sampler), Box<dyn std::error::Error>> {
    use wgpu::util::DeviceExt;

    let img  = image::open(path)?.to_rgba8();
    let (w, h) = img.dimensions();

    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label:           Some(label),
            size:            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: mip_levels(w, h),
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::Rgba8UnormSrgb,
            usage:           wgpu::TextureUsages::TEXTURE_BINDING
                           | wgpu::TextureUsages::COPY_DST
                           | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats:    &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &img,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        mip_level_count: Some(mip_levels(w, h)),
        ..Default::default()
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label:             Some(&format!("{label}_sampler")),
        address_mode_u:    wgpu::AddressMode::Repeat,
        address_mode_v:    wgpu::AddressMode::Repeat,
        address_mode_w:    wgpu::AddressMode::Repeat,
        mag_filter:        wgpu::FilterMode::Linear,
        min_filter:        wgpu::FilterMode::Linear,
        mipmap_filter:     wgpu::FilterMode::Linear,
        lod_min_clamp:     0.0,
        lod_max_clamp:     mip_levels(w, h) as f32,
        anisotropy_clamp:  16,
        ..Default::default()
    });

    log::info!("Loaded texture '{}' ({}×{})", label, w, h);
    Ok((texture, view, sampler))
}

fn mip_levels(w: u32, h: u32) -> u32 {
    (w.max(h) as f32).log2().floor() as u32 + 1
}

// ─── FPS Counter ─────────────────────────────────────────────────────────────

pub struct FpsCounter {
    frames:     u32,
    last_print: std::time::Instant,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self { frames: 0, last_print: std::time::Instant::now() }
    }

    /// Call once per frame. Logs FPS every second.
    pub fn tick(&mut self) {
        self.frames += 1;
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_print).as_secs_f32();
        if elapsed >= 1.0 {
            let fps = self.frames as f32 / elapsed;
            log::info!("FPS: {:.1}", fps);
            self.frames     = 0;
            self.last_print = now;
        }
    }
}

// ─── Delta-time stopwatch ────────────────────────────────────────────────────

pub struct Timer {
    last: std::time::Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self { last: std::time::Instant::now() }
    }

    /// Returns seconds elapsed since the last call to `tick`.
    pub fn tick(&mut self) -> f32 {
        let now = std::time::Instant::now();
        let dt  = now.duration_since(self.last).as_secs_f32();
        self.last = now;
        dt
    }
}
