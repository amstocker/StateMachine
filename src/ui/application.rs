use wgpu::util::StagingBelt;
use winit::{
    window::Window,
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopClosed},
    window::WindowBuilder,
};

use crate::ui::drawer::Drawer;


pub struct GPUState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub clear_color: wgpu::Color,
    pub staging_belt: StagingBelt,
}

impl GPUState {
    async fn init(window: &Window) -> Self {
        let size = window.inner_size();
        let clear_color = wgpu::Color::WHITE;

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let format = surface.get_supported_formats(&adapter)[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let staging_belt = wgpu::util::StagingBelt::new(1024);
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            staging_belt
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    fn render(&mut self, drawer: &mut Drawer) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            drawer.quad.draw_all(&mut render_pass);
        }

        drawer.text.draw_all(&mut encoder, view, self);

        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.staging_belt.recall();

        Ok(())
    }
}


pub trait ApplicationConfig {
    fn window_title(&self) -> &str;
}


pub trait Application: 'static + Sized {
    type Config: ApplicationConfig;

    fn init(config: Self::Config) -> Self;

    fn update(&mut self);

    fn draw(&self, drawer: &mut Drawer);

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window);

    fn run(config: Self::Config) {
        env_logger::init();
    
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(config.window_title())
            .build(&event_loop).unwrap();
   
        let mut state = pollster::block_on(GPUState::init(&window));

        let mut drawer = Drawer::init(&state.device, &state.config);
        
        let mut app = Self::init(config);
    
        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        },
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                            drawer.text.resize((physical_size.width, physical_size.height));
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                            drawer.text.resize((new_inner_size.width, new_inner_size.height));
                        },
                        _ => {},
                    };
                    app.handle_window_event(event, &window);
                },
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    app.draw(&mut drawer);
                    drawer.quad.write(&state.queue);
                    match state.render(&mut drawer) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                },
                Event::MainEventsCleared => {
                    app.update();
                    window.request_redraw();
                },
                _ => {}
            }
        });
    }
}


