use bytemuck::{cast_slice, Zeroable, Pod};
use wgpu::{TextureFormat, Device, include_wgsl, Buffer, RenderPipeline, RenderPass, Queue, Color, DepthStencilState};

use crate::sequencer::NUM_CHANNELS;

use crate::ui::{Depth, TransformInstance, Transform};
use crate::ui::util::color_to_f32_array;


const INSTANCE_BUFFER_SIZE: usize = 128;

pub struct Line {
    pub from: (f32, f32),
    pub to: (f32, f32),
    pub color: Color,
    pub depth: Depth
}

impl Line {
    pub fn instance_with_transform(&self, transform: Transform) -> LineInstance {
        let z = self.depth.z();
        LineInstance {
            from: [self.from.0, self.from.1, z],
            to: [self.to.0, self.to.1, z],
            color: color_to_f32_array(self.color),
            transform: transform.into()
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LineInstance {
    from: [f32; 3],
    to: [f32; 3],
    color: [f32; 4],
    transform: TransformInstance
}

impl LineInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<LineInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 10]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4
                }
            ],
        }
    }
}

pub struct LineHandler {
    render_pipeline: RenderPipeline,
    // instances: [LineInstance; INSTANCE_BUFFER_SIZE],
    instance_buffer: Buffer,
    instance_buffer_index: u32
}

impl LineHandler {
    pub fn init(device: &Device, format: TextureFormat, depth_stencil_state: DepthStencilState) -> LineHandler {
        use wgpu::util::DeviceExt;

        let shader = device.create_shader_module(include_wgsl!("line.wgsl"));

        let instances = [LineInstance::zeroed(); INSTANCE_BUFFER_SIZE];

        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Line Instance Buffer"),
                contents: cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    LineInstance::desc()
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
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: Some(depth_stencil_state),
            multisample: wgpu::MultisampleState::default(),
            multiview: None
        });

        Self {
            render_pipeline,
            // instances,
            instance_buffer,
            instance_buffer_index: 0
        }
    }

    pub fn write(&mut self, line: Line, transform: Transform, queue: &Queue) {
        let instance: LineInstance = line.instance_with_transform(transform);
        queue.write_buffer(
            &self.instance_buffer,
            (self.instance_buffer_index as u64) * (std::mem::size_of::<LineInstance>() as u64),
            cast_slice(&[instance])
        );
        self.instance_buffer_index += 1;
    }

    pub fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        render_pass.draw(
            0..2,
            0..self.instance_buffer_index
        );
        self.instance_buffer_index = 0;
    }
}


use crate::ui::render::Vertex;
const GRID_BACKGROUND_BUFFER_LENGTH: usize = 2 * (NUM_CHANNELS + 1);

fn create_grid_vertices() -> [Vertex; GRID_BACKGROUND_BUFFER_LENGTH] {
    let mut vertices = [Vertex::zeroed(); GRID_BACKGROUND_BUFFER_LENGTH];
    for i in 0..=NUM_CHANNELS {
        let y = -1.0 + 2.0 * (i as f32 / NUM_CHANNELS as f32);
        vertices[2 * i] = Vertex { position: [-1.0, y] };
        vertices[2 * i + 1] = Vertex { position: [1.0, y] };
    }
    vertices
}