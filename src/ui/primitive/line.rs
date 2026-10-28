use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, Color, DepthStencilState, Device, MultisampleState, Queue, RenderPass,
    RenderPipeline, TextureFormat,
};

use crate::ui::{Depth, Transform, TransformInstance};
use crate::util::color_to_f32_array;

use super::{instance_buffer::InstanceBuffer, RenderError};

pub struct Line {
    pub from: (f32, f32),
    pub to: (f32, f32),
    pub color: Color,
    pub depth: Depth,
}

impl Line {
    pub fn instance_with_transform(&self, transform: Transform) -> LineInstance {
        let z = self.depth.z();
        LineInstance {
            from: [self.from.0, self.from.1, z],
            to: [self.to.0, self.to.1, z],
            color: color_to_f32_array(self.color),
            transform: transform.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LineInstance {
    from: [f32; 3],
    to: [f32; 3],
    color: [f32; 4],
    transform: TransformInstance,
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
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct LineHandler {
    render_pipeline: RenderPipeline,
    instances: InstanceBuffer<LineInstance>,
}

impl LineHandler {
    pub fn init(
        device: &Device,
        format: TextureFormat,
        depth_stencil_state: DepthStencilState,
        multisample_state: MultisampleState,
    ) -> LineHandler {
        let shader = device.create_shader_module(include_wgsl!("line.wgsl"));

        let instances = InstanceBuffer::new(device, "Line Instance Buffer");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                buffers: &[LineInstance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(depth_stencil_state),
            multisample: multisample_state,
            multiview: None,
        });

        Self {
            render_pipeline,
            instances,
        }
    }

    pub fn write(&mut self, line: Line, transform: Transform) {
        self.instances.push(line.instance_with_transform(transform));
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) -> Result<(), RenderError> {
        self.instances.prepare(device, queue)
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }

    pub fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.instances.buffer.slice(..));
        render_pass.draw(0..2, 0..self.instances.len());
    }
}
