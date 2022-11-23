use wgpu::{util::StagingBelt, RenderPass};
use winit::{
    window::Window,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::ui::primitive::{QuadDrawer, TextDrawer};


pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub clear_color: wgpu::Color,
    pub staging_belt: StagingBelt,
}

impl State {
    async fn init(window: &Window) -> Self {
        let size = window.inner_size();
        let clear_color = wgpu::Color {
            r: 255.0 / 255.0,
            g: 250.0 / 255.0,
            b: 235.0 / 255.0,
            a: 1.0
        };

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
}


pub trait ApplicationConfig {
    fn window_title(&self) -> &str;
}

pub trait Application: 'static + Sized {
    type Config: ApplicationConfig;

    fn init(config: Self::Config, state: &State) -> Self;

    fn update(&mut self);

    fn draw(&self, quad_drawer: &mut QuadDrawer, text_drawer: &mut TextDrawer);

    fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>);

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window);

    fn run(config: Self::Config) {
        env_logger::init();
    
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(config.window_title())
            .build(&event_loop).unwrap();
   
        let mut state = pollster::block_on(State::init(&window));

        let mut quad_drawer = QuadDrawer::init(&state.device, state.config.format);
        let mut text_drawer = TextDrawer::init(&state.device, (state.config.width, state.config.height), state.config.format);
        
        let mut app = Self::init(config, &state);

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
                            text_drawer.resize((physical_size.width, physical_size.height));
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                            text_drawer.resize((new_inner_size.width, new_inner_size.height));
                        },
                        _ => {},
                    };
                    app.handle_window_event(event, &window);
                },
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    // Update application uniforms
                    app.draw(&mut quad_drawer, &mut text_drawer);
                    
                    // Write to buffer
                    quad_drawer.write(&state.queue);
                    
                    // Start render passes
                    let output = state.surface.get_current_texture().unwrap();
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                
                    let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });
                
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(state.clear_color),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    app.render(&mut render_pass);
                    quad_drawer.render(&mut render_pass);
                    
                    drop(render_pass);
                    text_drawer.render(&mut encoder, view, &mut state);
                
                    state.staging_belt.finish();
                    state.queue.submit(std::iter::once(encoder.finish()));
                    output.present();
                    state.staging_belt.recall();
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


