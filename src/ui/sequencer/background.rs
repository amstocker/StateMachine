use bytemuck::{cast_slice, Zeroable};
use wgpu::{TextureFormat, Device, include_wgsl, Buffer, RenderPipeline, RenderPass};

use crate::sequencer::NUM_CHANNELS;

use crate::ui::primitive::Vertex;


const BUFFER_LENGTH: usize = 2 * (NUM_CHANNELS + 1);

pub struct GridBackground {
    vertex_buffer: Buffer,
    render_pipeline: RenderPipeline
}

impl GridBackground {
    pub fn init(device: &Device, format: TextureFormat) -> GridBackground {
        use wgpu::util::DeviceExt;

        let shader = device.create_shader_module(include_wgsl!("grid.wgsl"));

        let vertices = create_grid_vertices();

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Grid Vertex Buffer"),
                contents: cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Grid Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()]
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None
        });

        Self {
            vertex_buffer,
            render_pipeline
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..(BUFFER_LENGTH as u32), 0..1);
    }
}

fn create_grid_vertices() -> [Vertex; BUFFER_LENGTH] {
    let mut vertices = [Vertex::zeroed(); BUFFER_LENGTH];
    for i in 0..=NUM_CHANNELS {
        let y = -1.0 + 2.0 * (i as f32 / NUM_CHANNELS as f32);
        vertices[2 * i] = Vertex { position: [-1.0, y] };
        vertices[2 * i + 1] = Vertex { position: [1.0, y] };
    }
    vertices
}