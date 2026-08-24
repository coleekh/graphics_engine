use winit::window::Window;

/// Holds the core wgpu objects.
pub struct GpuContext {
    pub instance:       wgpu::Instance,
    pub surface:        wgpu::Surface<'static>,
    pub adapter:        wgpu::Adapter,
    pub device:         wgpu::Device,
    pub queue:          wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl GpuContext {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // ── Instance ─────────────────────────────────────────────────────
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // ── Surface ──────────────────────────────────────────────────────
        // SAFETY: The window lives at least as long as the surface.
        let surface = unsafe {
            instance.create_surface_unsafe(
                wgpu::SurfaceTargetUnsafe::from_window(window)
                    .expect("Failed to create surface target"),
            )
        }.expect("Failed to create surface");

        // ── Adapter ──────────────────────────────────────────────────────
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference:       wgpu::PowerPreference::HighPerformance,
                compatible_surface:     Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No compatible GPU adapter found");

        log::info!("Adapter: {:?}", adapter.get_info().name);
        log::info!("Backend: {:?}", adapter.get_info().backend);

        // ── Device + Queue ────────────────────────────────────────────────
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("main_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // ── Surface configuration ─────────────────────────────────────────
        let caps   = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox   // low-latency triple buffer
        } else {
            wgpu::PresentMode::Fifo      // VSync
        };

        let surface_config = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width:        size.width.max(1),
            height:       size.height.max(1),
            present_mode,
            alpha_mode:   caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        log::info!(
            "Surface: {}x{} {:?} {:?}",
            surface_config.width,
            surface_config.height,
            surface_config.format,
            surface_config.present_mode,
        );

        Self { instance, surface, adapter, device, queue, surface_config }
    }
}
