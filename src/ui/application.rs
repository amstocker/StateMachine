use wgpu::{include_wgsl, util::StagingBelt, CommandEncoder, RenderPass, TextureView, SurfaceTexture};
use winit::{
    window::Window,
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopClosed},
    window::WindowBuilder,
};
use wgpu_glyph::{GlyphBrushBuilder, Section, Text, GlyphBrush};
use bytemuck::{cast_slice, Pod, Zeroable};

use crate::ui::fonts::*;


const TITLE: &str = "state_machine";

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2
];


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Mouse([f32; 2]);

impl Mouse {
    pub fn set(&mut self, (x, y): (f32, f32)) {
        self.0[0] = x;
        self.0[1] = y;
    }
}


struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    glyph_brush: GlyphBrush<()>,
    staging_belt: StagingBelt,
    mouse_position: Mouse,
    mouse_position_buffer: wgpu::Buffer,
    mouse_position_bind_group: wgpu::BindGroup
}

impl State {
    async fn new(window: &Window) -> Self {
        use wgpu::util::DeviceExt;

        let size = window.inner_size();
        let clear_color = wgpu::Color::BLACK;

        // GPU Init
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

        // Vertex Buffer
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let num_vertices = VERTICES.len() as u32;
        
        // Index Buffer
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;
        
        // Mouse Uniform
        let mouse_position = Mouse([0.0, 0.0]);
        let mouse_position_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Mouse Position Buffer"),
                contents: cast_slice(&[mouse_position]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let mouse_position_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("Mouse Position Bind Group Layout"),
        });
        let mouse_position_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mouse_position_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: mouse_position_buffer.as_entire_binding(),
                }
            ],
            label: Some("Mouse Position Bind Group"),
        });

        // Rendering Pipeline Init
        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &mouse_position_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None
        });

        // WGPU Glyph Init
        let staging_belt = wgpu::util::StagingBelt::new(1024);

        let font = JETBRAINS_MONO_LIGHT_ITALIC.into();
        let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, format);
        

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            index_buffer,
            num_indices,
            size,
            clear_color,
            glyph_brush,
            staging_belt,
            mouse_position,
            mouse_position_buffer,
            mouse_position_bind_group
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

    fn handle_window_event(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit
            },
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size)
            },
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(**new_inner_size)
            },
            WindowEvent::CursorMoved { position, .. } => {
                let x = position.x as f64 / self.size.width as f64;
                let y = position.y as f64 / self.size.height as f64;
                self.clear_color = wgpu::Color {
                    r: x,
                    g: y,
                    b: 1.0,
                    a: 1.0,
                };
                self.mouse_position.set((x as f32, y as f32));
                self.queue.write_buffer(&self.mouse_position_buffer, 0, cast_slice(&[self.mouse_position]));
            },
            _ => {},
        }
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.mouse_position_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.glyph_brush.queue(Section {
            screen_position: (30.0, 30.0),
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![Text::new("Hello wgpu_glyph!")
                .with_color([0.0, 0.0, 0.0, 1.0])
                .with_scale(40.0)],
            ..Section::default()
        });
        self.glyph_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder,
                &view,
                self.size.width,
                self.size.height,
            )
            .expect("Draw queued");
        
        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.staging_belt.recall();

        Ok(())
    }
}


#[derive(Clone)]
pub struct EventSender<E>(EventLoopProxy<E>) where E: 'static;

impl<E> EventSender<E> where E: 'static {
    pub fn send(&self, event: E) -> Result<(), EventLoopClosed<E>> {
        self.0.send_event(event)
    }
}

pub trait ApplicationConfig {
    fn window_title(&self) -> &str;
}


pub trait Application {
    type Event;
    type Config: ApplicationConfig;

    fn init(config: Self::Config, event_sender: EventSender<Self::Event>) -> Self;

    fn update(&mut self);

    fn draw(&self);

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window);

    fn handle_application_event(&mut self, event: Self::Event);

    fn run(config: Self::Config) where Self: 'static + Sized {
        env_logger::init();
    
        let event_loop: EventLoop<Self::Event> = EventLoopBuilder::with_user_event().build();
        let window = WindowBuilder::new()
            .with_title(config.window_title())
            .build(&event_loop).unwrap();
   
        let mut app = Self::init(config, EventSender(event_loop.create_proxy()));

        let mut state = pollster::block_on(State::new(&window));
    
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    state.handle_window_event(event, control_flow);
                    app.handle_window_event(event, &window);
                },
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    app.update();
                    // app.draw();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                },
                Event::MainEventsCleared => {
                    window.request_redraw();
                },
                Event::UserEvent(event) => {
                    app.handle_application_event(event)
                },
                _ => {}
            }
        });
    }
}


