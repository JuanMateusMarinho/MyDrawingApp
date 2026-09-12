use digital_canvas::{Canvas, Document, SelectionManager, Color, Rect, Vec2, EntityId, LayerId};
use anyhow::Result;
use std::sync::Arc;
use wgpu::{
    Device, Queue, Surface, SurfaceConfiguration, TextureFormat, TextureView,
    CommandEncoder, RenderPass, RenderPipeline, BindGroup, BindGroupLayout,
    Buffer, BufferUsages, VertexBufferLayout, VertexAttribute, VertexFormat,
    ShaderModule, PipelineLayout, Color as WgpuColor, LoadOp, StoreOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor, TextureViewDescriptor,
    Extent3d, TextureDimension, TextureUsages, FilterMode, AddressMode,
    SamplerDescriptor, SamplerBindingType, BindingType, BufferBindingType,
    ShaderStages, BindGroupLayoutEntry, BindingResource, BindGroupEntry,
    IndexFormat, PrimitiveTopology, PolygonMode, FrontFace, Face,
    MultisampleState, FragmentState, VertexState, PrimitiveState,
    DepthStencilState, ColorTargetState, BlendState, BlendComponent,
    BlendOperation, BlendFactor, ColorWrites,
    util::DeviceExt,
};
use winit::window::Window;
use bytemuck::{Pod, Zeroable};
use glam::{Vec2 as GlamVec2, Vec3 as GlamVec3, Vec4 as GlamVec4, Mat3, Mat4};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    canvas_size: [f32; 2],
    time: f32,
    _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct LayerUniforms {
    pub transform: [[f32; 4]; 4],
    pub opacity: f32,
    pub blend_mode: u32,
    pub visible: u32,
    pub _padding: [f32; 2],
}

pub struct Renderer {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    width: u32,
    height: u32,

    // Shaders
    layer_shader: ShaderModule,
    brush_shader: ShaderModule,
    ui_shader: ShaderModule,

    // Pipelines
    layer_pipeline: RenderPipeline,
    brush_pipeline: RenderPipeline,
    ui_pipeline: RenderPipeline,

    // Bind groups
    uniform_bind_group: BindGroup,
    uniform_buffer: Buffer,
    layer_bind_group_layout: BindGroupLayout,

    // Samplers
    linear_sampler: wgpu::Sampler,
    nearest_sampler: wgpu::Sampler,

    // Brush resources
    brush_texture: Option<wgpu::Texture>,
    brush_bind_group: Option<BindGroup>,

    // Canvas texture for offscreen rendering
    canvas_texture: wgpu::Texture,
    canvas_view: TextureView,
    canvas_bind_group: BindGroup,

    // UI resources
    ui_bind_group: BindGroup,
    ui_uniform_buffer: Buffer,
}

impl Renderer {
    pub fn new(window: Arc<Window>, settings: &digital_canvas::settings::Settings) -> Result<Self> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).ok_or_else(|| anyhow::anyhow!("Failed to find GPU adapter"))?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("DigitalCanvas Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shaders
        let layer_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Layer Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/layer.wgsl").into()),
        });

        let brush_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Brush Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/brush.wgsl").into()),
        });

        let ui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/ui.wgsl").into()),
        });

        // Create bind group layouts
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let layer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Layer Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let brush_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Brush Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create pipelines
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Layer Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &layer_bind_group_layout],
            push_constant_ranges: &[],
        });

        let layer_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Layer Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &layer_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &layer_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState {
                        color: BlendComponent {
                            operation: BlendOperation::Add,
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                        },
                        alpha: BlendComponent {
                            operation: BlendOperation::Add,
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(CullMode::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let brush_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Brush Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &brush_bind_group_layout],
            push_constant_ranges: &[],
        });

        let brush_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Brush Pipeline"),
            layout: Some(&brush_pipeline_layout),
            vertex: VertexState {
                module: &brush_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &brush_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(CullMode::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let ui_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&ui_pipeline_layout),
            vertex: VertexState {
                module: &ui_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &ui_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(CullMode::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create samplers
        let linear_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Linear Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        let nearest_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Nearest Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // Create canvas texture for offscreen rendering
        let canvas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let canvas_view = canvas_texture.create_view(&TextureViewDescriptor::default());

        let canvas_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Canvas Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let canvas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas Bind Group"),
            layout: &canvas_bind_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&canvas_view) },
                BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&linear_sampler) },
            ],
        });

        // UI uniform buffer
        let ui_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let ui_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: ui_uniform_buffer.as_entire_binding(),
            }],
        });

        Ok(Self {
            device,
            queue,
            surface,
            config,
            width,
            height,
            layer_shader,
            brush_shader,
            ui_shader,
            layer_pipeline,
            brush_pipeline,
            ui_pipeline,
            uniform_bind_group,
            uniform_buffer,
            layer_bind_group_layout,
            linear_sampler,
            nearest_sampler,
            brush_texture: None,
            brush_bind_group: None,
            canvas_texture,
            canvas_view,
            canvas_bind_group,
            ui_bind_group,
            ui_uniform_buffer,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.config.width = self.width;
        self.config.height = self.height;
        self.surface.configure(&self.device, &self.config);

        // Recreate canvas texture
        self.canvas_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size: Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        self.canvas_view = self.canvas_texture.create_view(&TextureViewDescriptor::default());
        self.canvas_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas Bind Group"),
            layout: &self.canvas_bind_group_layout(),
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&self.canvas_view) },
                BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&self.linear_sampler) },
            ],
        });
    }

    fn canvas_bind_group_layout(&self) -> wgpu::BindGroupLayout {
        self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Canvas Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    pub fn render(&mut self, canvas: &Canvas, document: &Document, selection: &SelectionManager) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Update uniforms
        let view_proj = Mat4::orthographic_rh(
            0.0, self.width as f32,
            self.height as f32, 0.0,
            -1.0, 1.0
        );
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            canvas_size: [self.width as f32, self.height as f32],
            time: 0.0,
            _padding: 0.0,
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Render to canvas texture first (offscreen)
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Canvas Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.canvas_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(WgpuColor { r: 0.1, g: 0.1, b: 0.12, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render layers
            self.render_layers(&mut render_pass, document);
        }

        // Render to screen
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Screen Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(WgpuColor { r: 0.15, g: 0.15, b: 0.18, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Draw canvas texture to screen
            render_pass.set_pipeline(&self.layer_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.canvas_bind_group, &[]);
            render_pass.draw(0..4, 0..1);

            // Draw selection overlay
            if selection.has_selection() {
                self.render_selection(&mut render_pass, selection);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn render_layers(&mut self, render_pass: &mut RenderPass, document: &Document) {
        render_pass.set_pipeline(&self.layer_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        for layer in document.layers() {
            if !layer.visible() {
                continue;
            }

            if let Some(texture) = layer.texture() {
                let layer_uniforms = LayerUniforms {
                    transform: layer.transform().to_matrix().to_cols_array_2d(),
                    opacity: layer.opacity(),
                    blend_mode: layer.blend_mode() as u32,
                    visible: 1,
                    _padding: [0.0, 0.0],
                };

                let layer_uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Layer Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[layer_uniforms]),
                    usage: BufferUsages::UNIFORM,
                });

                let layer_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Layer Bind Group"),
                    layout: &self.layer_bind_group_layout,
                    entries: &[
                        BindGroupEntry { binding: 0, resource: BindingResource::TextureView(texture.view()) },
                        BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&self.linear_sampler) },
                        BindGroupEntry { binding: 2, resource: layer_uniform_buffer.as_entire_binding() },
                    ],
                });

                render_pass.set_bind_group(1, &layer_bind_group, &[]);
                render_pass.draw(0..4, 0..1);
            }
        }
    }

    fn render_selection(&mut self, render_pass: &mut RenderPass, selection: &SelectionManager) {
        // Render marching ants or selection highlight
        // Implementation depends on selection type
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
