use wgpu::{util::DeviceExt};

pub struct WGPUBuffer {
    pub buffer: wgpu::Buffer
}

impl WGPUBuffer {
    fn new(device: &wgpu::Device, data: &[u8], usage: wgpu::BufferUsages, label: Option<&str>) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: label,
            contents: data,
            usage,
        });

        Self { buffer }
    }
}

pub trait BufferType {
    fn bgle(&self,  binding: u32) -> wgpu::BindGroupLayoutEntry;
    fn buffer(self) -> WGPUBuffer;
}

pub struct UniformBuffer {
    buffer: WGPUBuffer
}

impl UniformBuffer {
    pub fn new(device: &wgpu::Device, data: &[u8], label: Option<&str>) -> Self {
        Self {
            buffer: WGPUBuffer::new(device, data, wgpu::BufferUsages::UNIFORM, label)
        }
    }
}

impl BufferType for UniformBuffer {
    fn bgle(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        }
    }

    fn buffer(self) -> WGPUBuffer {
        self.buffer
    }
}

pub struct StorageBuffer {
    buffer: WGPUBuffer
}

impl StorageBuffer {
    pub fn new(device: &wgpu::Device, data: &[u8], label: Option<&str>) -> Self {
        Self {
            buffer: WGPUBuffer::new(device, data, wgpu::BufferUsages::STORAGE, label)
        }
    }
}

impl BufferType for StorageBuffer {
    fn bgle(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: true
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None
        }
    }

    fn buffer(self) -> WGPUBuffer {
        self.buffer
    }
}


pub struct WGPUBindGroup {
    buf_vec: std::vec::Vec<WGPUBuffer>,
    pub layout_entries_vec: std::vec::Vec<wgpu::BindGroupLayoutEntry>
}

impl WGPUBindGroup {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            buf_vec: std::vec::Vec::new(),
            layout_entries_vec: std::vec::Vec::new()
        }
    }

    pub fn add_buffer<T: BufferType>(&mut self, buffer: T) {
        self.layout_entries_vec.push(buffer.bgle(self.buf_vec.len() as u32));
        self.buf_vec.push(buffer.buffer());
    }

    pub fn get_bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &self.layout_entries_vec,
            label: None
        })
    }

    pub fn get_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let layout = self.get_bind_group_layout(device);

        let mut bind_group_entries = std::vec::Vec::new();

        for i in 0..self.buf_vec.len() {
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: i as u32,
                resource: self.buf_vec[i].buffer.as_entire_binding()
            })
        }

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &bind_group_entries,
            label: None
        })
    }
}