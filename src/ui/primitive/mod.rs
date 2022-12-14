mod line;
mod quad;
mod text;
mod vertex;

pub use line::*;
pub use quad::*;
pub use text::*;
pub use vertex::*;

use wgpu::*;
use wgpu::util::StagingBelt;
use winit::{window::Window, dpi::PhysicalSize};
use pollster::block_on;

use crate::ui::{Transform, Transformable, application::Style};


pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const MSAA_SAMPLE_COUNT: u32 = 4;


pub enum Primitive {
    Quad(Quad),
    Text(Text),
    Line(Line),
    Mesh
}


pub trait Drawable {
    fn draw(&self, draw: &mut Draw);
}

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    clear_color: Color,
    depth_buffer: TextureView,
    multisampled_framebuffer: TextureView,
    staging_belt: StagingBelt,
    quad_handler: QuadHandler,
    text_handler: TextHandler,
    line_handler: LineHandler
}

pub struct Draw<'r> {
    renderer: &'r mut Renderer,
    global_transform: Transform
}

impl<'r> Draw<'r> {
    pub fn primitive(&mut self, primitive: Primitive) {
        self.primitive_with_transform(primitive, self.global_transform);
    }

    pub fn line(&mut self, line: Line) {
        self.primitive(Primitive::Line(line));
    }

    pub fn quad(&mut self, quad: Quad) {
        self.primitive(Primitive::Quad(quad));
    }

    pub fn text(&mut self, text: Text) {
        self.primitive(Primitive::Text(text));
    }

    pub fn with<T: Transformable + Drawable>(&mut self, thing: &T) {
        let global_transform_copy = self.global_transform;
        self.push_transform(thing.transform());
        thing.draw(self);
        self.global_transform = global_transform_copy;
    }

    pub fn primitive_absolute(&mut self, primitive: Primitive) {
        self.primitive_with_transform(primitive, Transform::identity());
    }

    pub fn primitive_with_transform(&mut self, primitive: Primitive, transform: Transform) {
        let Renderer {
            queue,
            quad_handler,
            text_handler,
            line_handler,
            ..
        } = self.renderer;
        match primitive {
            Primitive::Quad(quad) => {
                quad_handler.write(quad, transform, queue)
            },
            Primitive::Text(ref text) => {
                text_handler.write(text, transform);
            },
            Primitive::Line(line) => {
                line_handler.write(line, transform, queue);
            },
            Primitive::Mesh => todo!(),
        }
    }

    pub fn text_length_to_width(&self, length: usize, scale: f32) -> f32 {
        self.renderer.text_handler.text_length_to_width(length, scale)
    }

    fn push_transform(&mut self, transform: Transform) {
        self.global_transform = self.global_transform.then(transform);
    }
}

impl Renderer {
    pub fn init(window: &Window, style: Style) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = block_on(instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )).unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                limits: Limits::default(),
                label: None,
            },
            None,
        )).unwrap();

        let format = surface.get_supported_formats(&adapter)[0];
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let staging_belt = StagingBelt::new(1024);

        let depth_buffer = create_depth_buffer(&device, size);
        let depth_stencil_state = DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Greater,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        };

        let multisampled_framebuffer = create_multisampled_framebuffer(&device, &config);
        let multisample_state = MultisampleState {
            count: MSAA_SAMPLE_COUNT,
            ..MultisampleState::default()
        };

        let quad_handler = QuadHandler::init(
            &device,
            format,
            depth_stencil_state.clone(),
            multisample_state.clone()
        );
        let text_handler = TextHandler::init(
            &device,
            format,
            depth_stencil_state.clone(),
            multisample_state.clone(),
            size
        );
        let line_handler = LineHandler::init(
            &device,
            format,
            depth_stencil_state,
            multisample_state
        );
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color: style.clear_color,
            depth_buffer,
            multisampled_framebuffer,
            staging_belt,
            quad_handler,
            text_handler,
            line_handler
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            self.depth_buffer = create_depth_buffer(&self.device, self.size);
            self.multisampled_framebuffer = create_multisampled_framebuffer(&self.device, &self.config);
            self.text_handler.resize(new_size);
        }
    }

    pub fn controller(&mut self) -> Draw {
        Draw {
            renderer: self,
            global_transform: Transform::identity()
        }
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
   
        
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.multisampled_framebuffer,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(self.clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_buffer,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(0.0),
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
            &self.multisampled_framebuffer,
            &self.depth_buffer,
            self.size.width,
            self.size.height
        );

        let final_pass =  encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Multisample Resolve Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.multisampled_framebuffer,
                resolve_target: Some(&view),
                ops: Operations {
                    load: LoadOp::Load,
                    store: false,
                },
            })],
            depth_stencil_attachment: None
        });
        drop(final_pass);
    
        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.staging_belt.recall();
    }
}


fn create_depth_buffer(
    device: &Device,
    size: PhysicalSize<u32>
) -> TextureView {
    let depth_texture = device.create_texture(&TextureDescriptor {
        label: Some("Depth Buffer"),
        size: Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT
    });

    depth_texture.create_view(&TextureViewDescriptor::default())
}


fn create_multisampled_framebuffer(
    device: &Device,
    config: &SurfaceConfiguration
) -> TextureView {
    let multisampled_texture = device.create_texture(&TextureDescriptor {
        label: Some("Multisampled Framebuffer"),
        size: Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: MSAA_SAMPLE_COUNT,
        dimension: TextureDimension::D2,
        format: config.format,
        usage: TextureUsages::RENDER_ATTACHMENT
    });

    multisampled_texture.create_view(&TextureViewDescriptor::default())
}