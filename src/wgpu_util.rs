use std::{default, mem};

use glam::{Mat4, Vec3};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{AccelerationStructureFlags, AccelerationStructureUpdateMode, BlasBuildEntry, BlasGeometries, BlasGeometrySizeDescriptors, BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor, BufferUsages, CreateBlasDescriptor, CreateTlasDescriptor, IndexFormat, TlasInstance, TlasPackage};

use winit::window::Window;
use winit::event::{WindowEvent};

use crate::wgpu_buffer::{UniformBuffer, BufferType};

pub struct WGPUState <'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    // pub render_pipeline: wgpu::ComputePipeline,
    // pub render_bind_group: wgpu::BindGroup,
    pub blit_pipeline: wgpu::RenderPipeline,
    pub blit_bind_group: wgpu::BindGroup,
    pub blit_storage_texture: wgpu::Texture,
    pub tlas_package: TlasPackage,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a Window,
}

impl<'a> WGPUState<'a>{
    pub async fn new(window: &'a Window) -> WGPUState<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), 
            //flags: wgpu::InstanceFlags::DEBUG,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::EXPERIMENTAL_RAY_TRACING_ACCELERATION_STRUCTURE
                    | wgpu::Features::EXPERIMENTAL_RAY_QUERY
                    | wgpu::Features::EXPERIMENTAL_RAY_HIT_VERTEX_RETURN
                    | wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::FLOAT32_FILTERABLE
                    | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits: wgpu::Limits {
                    max_binding_array_elements_per_shader_stage: 500000,
                    max_binding_array_sampler_elements_per_shader_stage: 1000,
                    max_buffer_size: 1024 * 1024 * 1024,
                    max_storage_buffer_binding_size: 1024 * 1024 * 1024,
                    ..Default::default()
                },
                label: None,
                ..Default::default()
            }
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // let render_pipeline = Self::create_render_pipeline(&device, &config, &queue);
        // let blit_pipeline = Self::create_blit_pipeline(&device, &config);

        // Create texture and sampler/bind groups for blit.wgsl
        let blit_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl for blit.wgsl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let storage_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 1.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let blit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor::preprocess_wgsl!("shaders/blit.wgsl").into()),
        });

        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout for blit.wgsl"),
            bind_group_layouts: &[&blit_bgl],
            push_constant_ranges: &[],
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline for blit.wgsl"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            multiview: None,
            cache: None,
        });

        let blit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group for blit.wgsl"),
            layout: &blit_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &storage_tex.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });


        // RT resources
        
        // let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     label: Some("bgl for shader.wgsl"),
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::COMPUTE,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         },
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: wgpu::ShaderStages::COMPUTE,
        //             ty: wgpu::BindingType::StorageTexture {
        //                 access: wgpu::StorageTextureAccess::WriteOnly,
        //                 format: wgpu::TextureFormat::Rgba8Unorm,
        //                 view_dimension: wgpu::TextureViewDimension::D2,
        //             },
        //             count: None,
        //         },
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 2,
        //             visibility: wgpu::ShaderStages::COMPUTE,
        //             ty: wgpu::BindingType::AccelerationStructure {

        //             },
        //             count: None,
        //         },
        //     ],
        // });
        
        // let vertices: [f32; 9] = [1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 0.0, -1.0, 0.0];

        // let indices: [u32; 3] = [0, 1, 2];

        // let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        //     label: Some("vertex buffer"),
        //     contents: bytemuck::cast_slice(&vertices),
        //     usage: BufferUsages::BLAS_INPUT,
        // });

        // let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        //     label: Some("index buffer"),
        //     contents: bytemuck::cast_slice(&indices),
        //     usage: BufferUsages::BLAS_INPUT,
        // });

        // let blas_size_desc = BlasTriangleGeometrySizeDescriptor {
        //     vertex_format: wgpu::VertexFormat::Float32x3,
        //     // 3 coordinates per vertex
        //     vertex_count: (vertices.len() / 3) as u32,
        //     index_format: Some(IndexFormat::Uint32),
        //     index_count: Some(indices.len() as u32),
        //     flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
        // };

        // let blas = device.create_blas(
        //     &CreateBlasDescriptor {
        //         label: None,
        //         flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
        //         update_mode: AccelerationStructureUpdateMode::Build,
        //     },
        //     BlasGeometrySizeDescriptors::Triangles {
        //         descriptors: vec![blas_size_desc.clone()],
        //     },
        // );

        // let tlas = device.create_tlas(&CreateTlasDescriptor {
        //     label: None,
        //     max_instances: 3,
        //     flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
        //     update_mode: AccelerationStructureUpdateMode::Build,
        // });

        // let mut tlas_package = TlasPackage::new(tlas);

        // tlas_package[0] = Some(TlasInstance::new(
        //     &blas,
        //     Mat4::from_translation(Vec3 {
        //         x: 0.0,
        //         y: 0.0,
        //         z: 0.0,
        //     })
        //     .transpose()
        //     .to_cols_array()[..12]
        //         .try_into()
        //         .unwrap(),
        //     0,
        //     0xff,
        // ));

        // let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // encoder.build_acceleration_structures(
        //     Some(&BlasBuildEntry {
        //         blas: &blas,
        //         geometry: BlasGeometries::TriangleGeometries(vec![BlasTriangleGeometry {
        //             size: &blas_size_desc,
        //             vertex_buffer: &vertex_buffer,
        //             first_vertex: 0,
        //             vertex_stride: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
        //             // in this case since one triangle gets no compression from an index buffer `index_buffer` and `first_index` could be `None`.
        //             index_buffer: Some(&index_buffer),
        //             first_index: Some(0),
        //             transform_buffer: None,
        //             transform_buffer_offset: None,
        //         }]),
        //     }),
        //     Some(&tlas_package),
        // );

        // queue.submit(Some(encoder.finish()));

        // let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("Shader"),
        //     source: wgpu::ShaderSource::Wgsl(include_preprocessed_wgsl!(std::fs::canonicalize(std::path::Path::new(file!()).parent().unwrap().join("shaders/shader_main.wgsl")).unwrap().to_str().unwrap()).into()),
        // });

        // let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: Some("Render Pipeline Layout"),
        //     bind_group_layouts: &[&bgl],
        //     push_constant_ranges: &[],
        // });

        // let render_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        //     label: Some("pipeline for shader.wgsl"),
        //     layout: Some(&render_pipeline_layout),
        //     module: &shader,
        //     entry_point: None,
        //     compilation_options: Default::default(),
        //     cache: None,
        // });

        // let uniform_buffer: UniformBuffer = UniformBuffer::new(&device, bytemuck::cast_slice(&[0.0, 0.0, 0.0]));

        // let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("bind group for shader.wgsl"),
        //     layout: &bgl,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: uniform_buffer.buffer().buffer.as_entire_binding(),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::TextureView(
        //                 &storage_tex.create_view(&wgpu::TextureViewDescriptor::default()),
        //             ),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 2,
        //             resource: wgpu::BindingResource::AccelerationStructure(tlas_package.tlas()),
        //         },
        //     ],
        // });

        let tlas = device.create_tlas(&wgpu::CreateTlasDescriptor {
            label: None,
            max_instances: 10000 as u32,
            flags: 
                wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE | 
                wgpu::AccelerationStructureFlags::ALLOW_RAY_HIT_VERTEX_RETURN,
                
            update_mode: wgpu::AccelerationStructureUpdateMode::Build,
        });

        let mut tlas_package = wgpu::TlasPackage::new(tlas);
                
        Self {
            surface,
            device,
            queue,
            config,
            // render_pipeline,
            // render_bind_group,
            blit_pipeline,
            blit_bind_group,
            blit_storage_texture: storage_tex,
            tlas_package: tlas_package,
            size,
            window
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {// && self.size != size {
            self.size = size;
            self.config.width = size.width;
            self.config.height = size.height;

            let blit_bgl = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bgl for blit.wgsl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

            let storage_tex = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                address_mode_u: Default::default(),
                address_mode_v: Default::default(),
                address_mode_w: Default::default(),
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 1.0,
                lod_max_clamp: 1.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            });

            let blit_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bind group for blit.wgsl"),
                layout: &blit_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &storage_tex.create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

            self.blit_bind_group = blit_bind_group;
            self.blit_storage_texture = storage_tex;

            self.surface.configure(&self.device, &self.config);
        }
    }

    // fn create_blit_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {


    //     blit_pipeline
    // }

    // fn create_render_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, queue: &wgpu::Queue) -> wgpu::ComputePipeline {
    //     // create resources


    // }
}

pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts(
            (p as *const T) as *const u8,
            ::core::mem::size_of::<T>(),
        )
    }
}