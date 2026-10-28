use bytemuck::{cast_slice, Pod};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

/// Reusable CPU collection plus one GPU upload per populated primitive batch.
pub(super) struct InstanceBuffer<T: Pod> {
    pub buffer: Buffer,
    pending: Vec<T>,
    capacity: usize,
    label: &'static str,
}

impl<T: Pod> InstanceBuffer<T> {
    pub fn new(device: &Device, label: &'static str) -> Self {
        let capacity = 128;
        Self {
            buffer: Self::allocate(device, label, capacity),
            pending: Vec::with_capacity(capacity),
            capacity,
            label,
        }
    }

    fn allocate(device: &Device, label: &'static str, capacity: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: (capacity * std::mem::size_of::<T>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn push(&mut self, instance: T) {
        self.pending.push(instance);
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) -> Result<(), super::RenderError> {
        let max_capacity = (device.limits().max_buffer_size / std::mem::size_of::<T>() as u64)
            .min(u32::MAX as u64) as usize;
        if self.pending.len() > max_capacity {
            return Err(super::RenderError::InstanceCapacityExceeded);
        }
        if self.pending.len() > self.capacity {
            self.capacity = self.pending.len().next_power_of_two().min(max_capacity);
            self.buffer = Self::allocate(device, self.label, self.capacity);
        }
        if !self.pending.is_empty() {
            queue.write_buffer(&self.buffer, 0, cast_slice(&self.pending));
        }
        Ok(())
    }

    pub fn len(&self) -> u32 {
        self.pending.len() as u32
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}
