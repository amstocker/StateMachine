use bytemuck::{Pod, Zeroable, cast_slice};
use wgpu::{Device, BindGroup};


#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct MousePosition([f32; 2]);

impl MousePosition {
    pub fn set(&mut self, (x, y): (f32, f32)) {
        self.0[0] = x;
        self.0[1] = y;
    }

    pub fn get(&self) -> (f32, f32) {
        (self.0[0], self.0[1])
    }

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