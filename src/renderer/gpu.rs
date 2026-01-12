use crate::math::mesh::{CurveMesh, ParametricSurfaceMesh, SurfaceMesh};
use crate::renderer::camera::{Camera, CameraUniform};
use crate::renderer::point_cloud::{PointCloudBuffers, point_2d_layout, point_3d_layout};

const MAX_SURFACE_VERTICES: usize = 500_000;
const MAX_SURFACE_INDICES: usize = 1_000_000;
const MAX_CURVE_VERTICES: usize = 10_000;
const MAX_GRID_VERTICES: usize = 2000;
const MAX_HEATMAP_VERTICES: usize = 500_000;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SurfaceUniforms {
    pub z_min: f32,
    pub z_max: f32,
    pub _pad1: f32,
    pub _pad2: f32,
}

pub struct MathBuffers {
    pub surface_vertex_buffer: wgpu::Buffer,
    pub surface_normal_buffer: wgpu::Buffer,
    pub surface_index_buffer: wgpu::Buffer,
    pub surface_vertex_count: u32,
    pub surface_index_count: u32,

    pub curve_vertex_buffer: wgpu::Buffer,
    pub curve_vertex_count: u32,

    pub grid_vertex_buffer: wgpu::Buffer,
    pub grid_vertex_count: u32,

    pub surface_uniform_buffer: wgpu::Buffer,

    pub heatmap_buffer: wgpu::Buffer,
    pub heatmap_vertex_count: u32,

    pub curve_2d_buffer: wgpu::Buffer,
    pub curve_2d_vertex_count: u32,

    pub z_min: f32,
    pub z_max: f32,
}

impl MathBuffers {
    pub fn new(device: &wgpu::Device) -> Self {
        let surface_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surface Vertex Buffer"),
            size: (MAX_SURFACE_VERTICES * 3 * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let surface_normal_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surface Normal Buffer"),
            size: (MAX_SURFACE_VERTICES * 3 * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let surface_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surface Index Buffer"),
            size: (MAX_SURFACE_INDICES * 4) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let curve_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Curve Vertex Buffer"),
            size: (MAX_CURVE_VERTICES * 3 * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grid_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Vertex Buffer"),
            size: (MAX_GRID_VERTICES * 3 * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let surface_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Surface Uniform Buffer"),
            size: std::mem::size_of::<SurfaceUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let heatmap_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Heatmap Buffer"),
            size: (MAX_HEATMAP_VERTICES * 3 * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let curve_2d_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Curve 2D Buffer"),
            size: (MAX_CURVE_VERTICES * 2 * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface_vertex_buffer,
            surface_normal_buffer,
            surface_index_buffer,
            surface_vertex_count: 0,
            surface_index_count: 0,
            curve_vertex_buffer,
            curve_vertex_count: 0,
            grid_vertex_buffer,
            grid_vertex_count: 0,
            surface_uniform_buffer,
            heatmap_buffer,
            heatmap_vertex_count: 0,
            curve_2d_buffer,
            curve_2d_vertex_count: 0,
            z_min: 0.0,
            z_max: 1.0,
        }
    }

    pub fn upload_surface(&mut self, queue: &wgpu::Queue, mesh: &SurfaceMesh) {
        let vertex_count = mesh.mesh.vertices.len().min(MAX_SURFACE_VERTICES * 3);
        let index_count = mesh.mesh.indices.len().min(MAX_SURFACE_INDICES);

        queue.write_buffer(
            &self.surface_vertex_buffer,
            0,
            bytemuck::cast_slice(&mesh.mesh.vertices[..vertex_count]),
        );
        queue.write_buffer(
            &self.surface_normal_buffer,
            0,
            bytemuck::cast_slice(&mesh.mesh.normals[..vertex_count]),
        );
        queue.write_buffer(
            &self.surface_index_buffer,
            0,
            bytemuck::cast_slice(&mesh.mesh.indices[..index_count]),
        );

        self.surface_vertex_count = (vertex_count / 3) as u32;
        self.surface_index_count = index_count as u32;
        self.z_min = mesh.z_min;
        self.z_max = mesh.z_max;

        let uniforms = SurfaceUniforms {
            z_min: mesh.z_min,
            z_max: mesh.z_max,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        queue.write_buffer(
            &self.surface_uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
    }

    pub fn upload_heatmap(&mut self, queue: &wgpu::Queue, data: &[f32]) {
        let count = data.len().min(MAX_HEATMAP_VERTICES * 3);
        queue.write_buffer(
            &self.heatmap_buffer,
            0,
            bytemuck::cast_slice(&data[..count]),
        );
        self.heatmap_vertex_count = (count / 3) as u32;
    }

    pub fn upload_curve_2d(&mut self, queue: &wgpu::Queue, data: &[f32]) {
        let count = data.len().min(MAX_CURVE_VERTICES * 2);
        queue.write_buffer(
            &self.curve_2d_buffer,
            0,
            bytemuck::cast_slice(&data[..count]),
        );
        self.curve_2d_vertex_count = (count / 2) as u32;
    }

    pub fn upload_curve(&mut self, queue: &wgpu::Queue, mesh: &CurveMesh) {
        let vertex_count = mesh.vertices.len().min(MAX_CURVE_VERTICES * 3);
        queue.write_buffer(
            &self.curve_vertex_buffer,
            0,
            bytemuck::cast_slice(&mesh.vertices[..vertex_count]),
        );
        self.curve_vertex_count = (vertex_count / 3) as u32;
    }

    pub fn upload_parametric_surface(&mut self, queue: &wgpu::Queue, mesh: &ParametricSurfaceMesh) {
        let vertex_count = mesh.mesh.vertices.len().min(MAX_SURFACE_VERTICES * 3);
        let index_count = mesh.mesh.indices.len().min(MAX_SURFACE_INDICES);

        queue.write_buffer(
            &self.surface_vertex_buffer,
            0,
            bytemuck::cast_slice(&mesh.mesh.vertices[..vertex_count]),
        );
        queue.write_buffer(
            &self.surface_normal_buffer,
            0,
            bytemuck::cast_slice(&mesh.mesh.normals[..vertex_count]),
        );
        queue.write_buffer(
            &self.surface_index_buffer,
            0,
            bytemuck::cast_slice(&mesh.mesh.indices[..index_count]),
        );

        self.surface_vertex_count = (vertex_count / 3) as u32;
        self.surface_index_count = index_count as u32;
        self.z_min = 0.0;
        self.z_max = 1.0;

        let uniforms = SurfaceUniforms {
            z_min: 0.0,
            z_max: 1.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        queue.write_buffer(
            &self.surface_uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
    }

    pub fn upload_grid(&mut self, queue: &wgpu::Queue, vertices: &[f32]) {
        let vertex_count = vertices.len().min(MAX_GRID_VERTICES * 3);
        queue.write_buffer(
            &self.grid_vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices[..vertex_count]),
        );
        self.grid_vertex_count = (vertex_count / 3) as u32;
    }
}

pub struct GpuState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    pub pipeline_3d: wgpu::RenderPipeline,
    pub pipeline_2d: wgpu::RenderPipeline,
    pub pipeline_surface: wgpu::RenderPipeline,
    pub pipeline_curve: wgpu::RenderPipeline,
    pub pipeline_grid: wgpu::RenderPipeline,
    pub pipeline_math_2d: wgpu::RenderPipeline,
    pub pipeline_curve_2d: wgpu::RenderPipeline,

    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub math_bind_group: wgpu::BindGroup,

    pub point_buffers: PointCloudBuffers,
    pub math_buffers: MathBuffers,

    pub depth_texture: wgpu::TextureView,
}

fn surface_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 12,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        }],
    }
}

fn surface_normal_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 12,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x3,
        }],
    }
}

fn heatmap_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 12,
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

fn curve_2d_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
        }],
    }
}

impl GpuState {
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let math_buffers = MathBuffers::new(&device);

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let math_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Math Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let math_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Math Bind Group"),
            layout: &math_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: math_buffers.surface_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout_3d = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("3D Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline_3d = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Render Pipeline"),
            layout: Some(&pipeline_layout_3d),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[point_3d_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pipeline_layout_2d = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline_2d = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D Render Pipeline"),
            layout: Some(&pipeline_layout_2d),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_2d_main"),
                buffers: &[point_2d_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_2d_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pipeline_layout_math = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Math Pipeline Layout"),
            bind_group_layouts: &[&math_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline_surface = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Surface Render Pipeline"),
            layout: Some(&pipeline_layout_math),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_surface_main"),
                buffers: &[surface_vertex_layout(), surface_normal_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_surface_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pipeline_curve = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Curve Render Pipeline"),
            layout: Some(&pipeline_layout_3d),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_curve_main"),
                buffers: &[surface_vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_curve_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pipeline_grid = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Render Pipeline"),
            layout: Some(&pipeline_layout_3d),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_grid_main"),
                buffers: &[surface_vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_grid_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pipeline_math_2d = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Math 2D Pipeline"),
            layout: Some(&pipeline_layout_2d),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_math_2d_main"),
                buffers: &[heatmap_vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_math_2d_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pipeline_curve_2d = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Curve 2D Pipeline"),
            layout: Some(&pipeline_layout_2d),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_curve_2d_main"),
                buffers: &[curve_2d_vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_curve_2d_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let point_buffers = PointCloudBuffers::new(&device);
        let depth_texture = Self::create_depth_texture(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline_3d,
            pipeline_2d,
            pipeline_surface,
            pipeline_curve,
            pipeline_grid,
            pipeline_math_2d,
            pipeline_curve_2d,
            camera_buffer,
            camera_bind_group,
            math_bind_group,
            point_buffers,
            math_buffers,
            depth_texture,
        }
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Self::create_depth_texture(&self.device, &self.config);
        }
    }

    pub fn update_camera(&self, camera: &Camera) {
        let uniform = CameraUniform::from_camera(camera);
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn set_vsync(&mut self, enabled: bool) {
        self.config.present_mode = if enabled {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render_3d(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("3D Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline_3d);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.point_buffers.current_3d_buffer().slice(..));
        render_pass.draw(0..self.point_buffers.points_count_3d(), 0..1);
    }

    pub fn render_2d(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("2D Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline_2d);
        render_pass.set_vertex_buffer(0, self.point_buffers.current_2d_buffer().slice(..));
        render_pass.draw(0..self.point_buffers.points_count_2d(), 0..1);
    }

    pub fn render_surface(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surface Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline_surface);
        render_pass.set_bind_group(0, &self.math_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.math_buffers.surface_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.math_buffers.surface_normal_buffer.slice(..));
        render_pass.set_index_buffer(
            self.math_buffers.surface_index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..self.math_buffers.surface_index_count, 0, 0..1);
    }

    pub fn render_surface_no_clear(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Surface Render Pass (No Clear)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline_surface);
        render_pass.set_bind_group(0, &self.math_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.math_buffers.surface_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.math_buffers.surface_normal_buffer.slice(..));
        render_pass.set_index_buffer(
            self.math_buffers.surface_index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..self.math_buffers.surface_index_count, 0, 0..1);
    }

    pub fn render_curve(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Curve Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline_curve);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.math_buffers.curve_vertex_buffer.slice(..));
        render_pass.draw(0..self.math_buffers.curve_vertex_count, 0..1);
    }

    pub fn render_curve_no_clear(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Curve Render Pass (No Clear)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline_curve);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.math_buffers.curve_vertex_buffer.slice(..));
        render_pass.draw(0..self.math_buffers.curve_vertex_count, 0..1);
    }

    pub fn render_grid(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        clear: bool,
    ) {
        let load_op = if clear {
            wgpu::LoadOp::Clear(wgpu::Color::BLACK)
        } else {
            wgpu::LoadOp::Load
        };
        let depth_load = if clear {
            wgpu::LoadOp::Clear(1.0)
        } else {
            wgpu::LoadOp::Load
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Grid Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline_grid);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.math_buffers.grid_vertex_buffer.slice(..));
        render_pass.draw(0..self.math_buffers.grid_vertex_count, 0..1);
    }

    pub fn render_math_2d(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Math 2D Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline_math_2d);
        render_pass.set_vertex_buffer(0, self.math_buffers.heatmap_buffer.slice(..));
        render_pass.draw(0..self.math_buffers.heatmap_vertex_count, 0..1);
    }

    pub fn render_curve_2d(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Curve 2D Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline_curve_2d);
        render_pass.set_vertex_buffer(0, self.math_buffers.curve_2d_buffer.slice(..));
        render_pass.draw(0..self.math_buffers.curve_2d_vertex_count, 0..1);
    }
}

pub fn generate_grid_vertices(size: f32, divisions: u32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let step = size * 2.0 / divisions as f32;
    let half = size;

    for i in 0..=divisions {
        let pos = -half + i as f32 * step;
        vertices.extend_from_slice(&[pos, 0.0, -half, pos, 0.0, half]);
        vertices.extend_from_slice(&[-half, 0.0, pos, half, 0.0, pos]);
    }

    vertices.extend_from_slice(&[-half, 0.0, 0.0, half, 0.0, 0.0]);
    vertices.extend_from_slice(&[0.0, 0.0, -half, 0.0, 0.0, half]);
    vertices.extend_from_slice(&[0.0, -half, 0.0, 0.0, half, 0.0]);

    vertices
}
