mod quad;
mod text;
mod line;
mod vertex;

pub use quad::*;
pub use text::*;
pub use line::*;
pub use vertex::*;

use wgpu::{util::StagingBelt, TextureView, DepthStencilState};
use winit::window::Window;


pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub enum Primitive {
    Quad(Quad),
    Text(Text),
    Line(Line),
    Mesh
}


pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    depth_buffer: TextureView,
    staging_belt: StagingBelt,
    quad_handler: QuadHandler,
    text_handler: TextHandler,
    line_handler: LineHandler
}

pub struct RendererController<'r> {
    renderer: &'r mut Renderer
}

impl<'r> RendererController<'r> {
    pub fn draw(&mut self, primitive: Primitive) {
        let Renderer {
            queue,
            quad_handler,
            text_handler,
            line_handler,
            ..
        } = self.renderer;
        match primitive {
            Primitive::Quad(quad) => {
                quad_handler.write(quad, queue)
            },
            Primitive::Text(ref text) => {
                text_handler.write(text);
            },
            Primitive::Line(line) => {
                line_handler.write(line, queue);
            },
            Primitive::Mesh => todo!(),
        }
    }

    pub fn set_transform(&mut self) {
        todo!();

        // Set the "global transform",
        //  might need to store in instance buffer...
    }
}

impl Renderer {
    pub fn init(window: &Window) -> Self {
        let size = window.inner_size();
        let clear_color = wgpu::Color {
            r: 255.0 / 255.0,
            g: 250.0 / 255.0,
            b: 235.0 / 255.0,
            a: 1.0
        };

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )).unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )).unwrap();

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

        let depth_buffer = create_depth_buffer(&device, size);
        let depth_stencil_state = wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let staging_belt = wgpu::util::StagingBelt::new(1024);

        let quad_handler = QuadHandler::init(&device, format, depth_stencil_state.clone());
        let text_handler = TextHandler::init(&device, format, depth_stencil_state.clone(), size);
        let line_handler = LineHandler::init(&device, format, depth_stencil_state);
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            depth_buffer,
            staging_belt,
            quad_handler,
            text_handler,
            line_handler
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            self.depth_buffer = create_depth_buffer(&self.device, self.size);
            self.text_handler.resize(new_size);
        }
    }

    pub fn controller(&mut self) -> RendererController {
        RendererController { renderer: self }
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
    
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &&self.depth_buffer,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        self.quad_handler.render(&mut render_pass);
        self.line_handler.render(&mut render_pass);
        drop(render_pass);
        
        self.text_handler.render(
            &self.device,
            &mut self.staging_belt,
            &mut encoder,
            &view,
            &self.depth_buffer,
            self.size.width,
            self.size.height
        );
    
        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.staging_belt.recall();
    }
}


fn create_depth_buffer(device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) -> wgpu::TextureView {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Buffer"),
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    });

    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}