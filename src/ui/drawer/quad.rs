use bytemuck::{Pod, Zeroable, cast_slice};
use wgpu::{include_wgsl, ShaderModule, Device, RenderPipeline, Buffer, RenderPass, TextureFormat, Queue, Color};

use crate::ui::mouse::MousePosition;

use super::util::color_to_f32_array;


pub const MAX_QUADS: usize = 128;

const QUAD_VERTICES: &[QuadVertex] = &[
    QuadVertex { position: [0.0, 0.0] },
    QuadVertex { position: [1.0, 0.0] },
    QuadVertex { position: [0.0, 1.0] },
    QuadVertex { position: [1.0, 1.0] }
];

const QUAD_INDICES: &[u16] = &[
    0, 1, 2,
    3, 2, 1
];

const NUM_QUAD_INDICES: u32 = QUAD_INDICES.len() as u32;


#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct QuadVertex {
    position: [f32; 2]
}

impl QuadVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct QuadInstance {
    position: [f32; 3],
    size: [f32; 2],
    color: [f32; 4]
}

impl QuadInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<QuadInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                }
            ],
        }
    }
}

pub struct Quad {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub color: Color,
    pub z: f32
}

impl Quad {
    pub fn contains(&self, position: MousePosition) -> bool {
        position.x > self.position.0 &&
        position.x < self.position.0 + self.size.0 &&
        position.y > self.position.1 &&
        position.y < self.position.1 + self.size.1
    }
}

impl Into<QuadInstance> for &Quad {
    fn into(self) -> QuadInstance {
        QuadInstance {
            position: [self.position.0, self.position.1, self.z],
            size: [self.size.0, self.size.1],
            color: color_to_f32_array(self.color)
        }
    }
}

pub struct QuadDrawer {
    shader: ShaderModule,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: Buffer,
    render_pipeline: RenderPipeline,
    instances: [QuadInstance; MAX_QUADS],
    active: u32
}

impl QuadDrawer {
    pub fn init(device: &Device, format: TextureFormat) -> Self {
        use wgpu::util::DeviceExt;

        let shader = device.create_shader_module(include_wgsl!("quad.wgsl"));

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Quad Vertex Buffer"),
                contents: cast_slice(QUAD_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Quad Index Buffer"),
                contents: cast_slice(QUAD_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let instances = [QuadInstance::zeroed(); MAX_QUADS];
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Quad Instance Buffer"),
                contents: cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Quad Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    QuadVertex::desc(),
                    QuadInstance::desc()
                ]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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

        Self {
            shader,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            render_pipeline,
            instances,
            active: 0
        }
    }

    pub fn draw(&mut self, quad: &Quad) {
        self.instances[self.active as usize] = quad.into();
        self.active += 1;
    }

    pub fn write(&self, queue: &Queue) {
        queue.write_buffer(&self.instance_buffer, 0, cast_slice(&[self.instances]));
    }

    pub fn draw_all<'a>(&'a mut self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            self.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16
        );
        render_pass.draw_indexed(
            0..NUM_QUAD_INDICES,
            0,
            0..self.active
        );
        self.active = 0;
    }
}