use winit::window::Window;

/// Holds the core wgpu objects.
pub struct GpuContext<'a> {
    pub instance:       wgpu::Instance,
    pub surface:        wgpu::Surface<'a>,
    pub adapter:        wgpu::Adapter,
    pub device:         wgpu::Device,
    pub queue:          wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl<'a> GpuContext<'a> {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(
        wgpu::SurfaceTarget::from(window)
        ).expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference:       wgpu::PowerPreference::HighPerformance,
                compatible_surface:     Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No compatible GPU adapter found");

        let adapter_info = adapter.get_info();
        log::info!("Adapter: {}", adapter_info.name);
        log::info!("Backend: {:?}", adapter_info.backend);
        log::info!("Device Type: {:?}", adapter_info.device_type);
        log::debug!("Driver: {}", adapter_info.driver);
        log::debug!("Driver Info: {}", adapter_info.driver_info);

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
