use bytemuck::{Pod, Zeroable, cast_slice};
use wgpu::{Device, BindGroup};
use winit::dpi::{PhysicalPosition, PhysicalSize};


#[derive(Default, Clone, Copy)]
pub struct MousePosition {
    pub x: f32,
    pub y: f32
}

impl MousePosition {
    pub fn from_physical(position: &PhysicalPosition<f64>, size: PhysicalSize<u32>) -> MousePosition {
        MousePosition {
            x: position.x as f32 / size.width as f32,
            y: (size.height as f32 - position.y as f32) / size.height as f32
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

impl MousePositionUniform {
    pub fn bind_group(&self, device: &Device) -> BindGroup {
        use wgpu::util::DeviceExt;

        let mouse_position_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Mouse Position Buffer"),
                contents: cast_slice(&[self.0]),
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

        mouse_position_bind_group
    }
}