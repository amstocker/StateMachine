use bytemuck::{Pod, Zeroable, cast_slice};
use wgpu::{Device, BindGroupLayoutEntry, BindGroupEntry, Buffer, Queue};
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::ui::Transform;


#[derive(Debug, Default)]
pub struct Mouse {
    pub position: MousePosition,
    pub position_delta: (f32, f32)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MousePosition {
    pub x: f32,
    pub y: f32,
}

impl MousePosition {
    pub fn delta(&self, last: MousePosition) -> (f32, f32) {
        (
            self.x - last.x,
            self.y - last.y
        )
    }

    pub fn from_physical(position: &PhysicalPosition<f64>, size: PhysicalSize<u32>) -> MousePosition {
        MousePosition {
            x: position.x as f32 / size.width as f32,
            y: position.y as f32 / size.height as f32
        }
    }

    pub fn transform(&self, transform: Transform) -> MousePosition {
        MousePosition { 
            x: transform.scale.0 * self.x + transform.translate.0,
            y: transform.scale.1 * self.y + transform.translate.1
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct MousePositionUniform([f32; 2]);

impl Into<MousePositionUniform> for MousePosition {
    fn into(self) -> MousePositionUniform {
        MousePositionUniform([self.x, self.y])
    }
}

pub struct MousePositionUniformBuffer {
    uniform: MousePositionUniform,
    buffer: Buffer
}

impl MousePositionUniformBuffer {
    pub fn new(device: &Device) -> Self {
        use wgpu::util::DeviceExt;

        let uniform = MousePositionUniform([0.0, 0.0]);
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Mouse Position Buffer"),
                contents: cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self {
            uniform,
            buffer
        }
    }

    pub fn write(&self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, cast_slice(&[self.uniform]));
    }

    pub fn bind_group_entry(&self, device: &Device, binding: u32) -> (BindGroupLayoutEntry, BindGroupEntry) {
        let bind_group_layout_entry = BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bing_group_entry = BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        };

        (bind_group_layout_entry, bing_group_entry)
    }
}