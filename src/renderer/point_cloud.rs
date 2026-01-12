use bytemuck::{Pod, Zeroable};

const NUM_BUFFERS: usize = 3;
const MAX_POINTS_PER_BUFFER: usize = 10_000_000;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Point3D {
    pub position: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Point2D {
    pub position: [f32; 2],
    pub value: f32,
}

pub struct PointCloudBuffers {
    buffers_3d: [wgpu::Buffer; NUM_BUFFERS],
    buffers_2d: [wgpu::Buffer; NUM_BUFFERS],

    current_buffer: usize,
    points_count_3d: usize,
    points_count_2d: usize,
}

impl PointCloudBuffers {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffers_3d = std::array::from_fn(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Point Cloud 3D Buffer"),
                size: (MAX_POINTS_PER_BUFFER * std::mem::size_of::<Point3D>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        let buffers_2d = std::array::from_fn(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Point Cloud 2D Buffer"),
                size: (MAX_POINTS_PER_BUFFER * std::mem::size_of::<Point2D>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        Self {
            buffers_3d,
            buffers_2d,
            current_buffer: 0,
            points_count_3d: 0,
            points_count_2d: 0,
        }
    }

    pub fn upload_3d(&mut self, queue: &wgpu::Queue, points: &[f32]) {
        if points.is_empty() {
            return;
        }

        let next_buffer = (self.current_buffer + 1) % NUM_BUFFERS;
        let point_count = points.len() / 3;
        let point_count = point_count.min(MAX_POINTS_PER_BUFFER);

        let byte_len = point_count * std::mem::size_of::<Point3D>();
        queue.write_buffer(
            &self.buffers_3d[next_buffer],
            0,
            &bytemuck::cast_slice(points)[..byte_len],
        );

        self.current_buffer = next_buffer;
        self.points_count_3d = point_count;
    }

    pub fn upload_2d(&mut self, queue: &wgpu::Queue, points: &[f32]) {
        if points.is_empty() {
            return;
        }

        let next_buffer = (self.current_buffer + 1) % NUM_BUFFERS;
        let point_count = points.len() / 3;
        let point_count = point_count.min(MAX_POINTS_PER_BUFFER);

        let byte_len = point_count * std::mem::size_of::<Point2D>();
        queue.write_buffer(
            &self.buffers_2d[next_buffer],
            0,
            &bytemuck::cast_slice(points)[..byte_len],
        );

        self.current_buffer = next_buffer;
        self.points_count_2d = point_count;
    }

    pub fn current_3d_buffer(&self) -> &wgpu::Buffer {
        &self.buffers_3d[self.current_buffer]
    }

    pub fn current_2d_buffer(&self) -> &wgpu::Buffer {
        &self.buffers_2d[self.current_buffer]
    }

    pub fn points_count_3d(&self) -> u32 {
        self.points_count_3d as u32
    }

    pub fn points_count_2d(&self) -> u32 {
        self.points_count_2d as u32
    }
}

pub fn point_3d_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Point3D>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        }],
    }
}

pub fn point_2d_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Point2D>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: 8,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32,
            },
        ],
    }
}
