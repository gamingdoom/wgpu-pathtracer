use std::num::NonZeroU32;

use glam::{Vec3, Mat4};
use wgpu::{util::DeviceExt, TlasInstance};

use crate::{scene, shaders::shader_definitions, uniforms, wgpu_buffer::{BufferType, StorageBuffer, UniformBuffer}, wgpu_util};
use crate::render_steps::RenderStep;

pub struct RTStep {
    pipeline: wgpu::ComputePipeline,
    bind_groups: Vec<wgpu::BindGroup>,
    
    blases: Vec<wgpu::Blas>,
    tlas_package: wgpu::TlasPackage,
    pub output_texture_view: wgpu::TextureView,
    
    rt_bind_group: Option<wgpu::BindGroup>,
    texture_bind_group: Option<wgpu::BindGroup>
}

impl RTStep {
    fn get_rt_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl for shader_main.wgsl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::AccelerationStructure {
                        vertex_return: true
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        bgl
    }

    fn get_texture_bgl(device: &wgpu::Device, scene: &scene::Scene) -> wgpu::BindGroupLayout {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl for shader_main.wgsl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(scene.textures.len() as u32),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    
                    // Should be same as number of texture groups (diffuse + normal + ...)
                    count: NonZeroU32::new(scene.textures.len() as u32),
                },
            ],
        });

        bgl
    }

    fn get_uniform_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl for shader_main.wgsl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        bgl
    }

    pub fn set_output_texture(&mut self, tex: &wgpu::Texture) {
        self.output_texture_view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn create_static_bind_groups(&mut self, state: &wgpu_util::WGPUState, scene: &scene::Scene) {
        let mut uniforms = uniforms::Uniforms::new(&scene.camera);

        let materials_uniform_vec = scene.materials.iter().map(|m| uniforms::Material::new(m.clone())).collect::<Vec<uniforms::Material>>();

        let materials_buffer = StorageBuffer::new(&state.rt_device, bytemuck::cast_slice(&materials_uniform_vec), Some("Materials"));

        let vertices_buffer = StorageBuffer::new(&state.rt_device, bytemuck::cast_slice(&scene.vertices), Some("Vertices"));
        let indices_buffer = StorageBuffer::new(&state.rt_device, bytemuck::cast_slice(&scene.indices), Some("Indices"));

        // InstanceInfo
        let mut instance_infos = Vec::new();
        for mesh in &scene.meshes {
            instance_infos.push(uniforms::InstanceInfo::new(mesh.index_offset));
        }

        let instance_info_buffer = StorageBuffer::new(&state.rt_device, bytemuck::cast_slice(&instance_infos), Some("InstanceInfos"));

        let mut light_triangles = Vec::new();
        for (i, indices) in scene.indices.chunks(3).enumerate() {
            // Get the material index from self.meshes's offsets
            let mut mat_idx = 0;
            for mesh in &scene.meshes {
                if ((i * 3) as u32) >= mesh.index_offset && ((i * 3) as u32) < mesh.index_offset + mesh.index_count {
                    mat_idx = mesh.material_index;
                    break;
                }
            }

            let emissive = scene.materials[mat_idx as usize].emissive_texture_idx != 4;
            if !emissive {
                continue;
            }

            light_triangles.push(
                uniforms::Triangle::new(
                    scene.vertices[indices[0].index as usize].position,
                    scene.vertices[indices[1].index as usize].position,
                    scene.vertices[indices[2].index as usize].position,
                    scene.vertices[indices[0].index as usize].normal,
                )
            );
        }

        if light_triangles.len() == 0 {
            light_triangles = vec![uniforms::Triangle::new(
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
            )];
        }

        let light_triangles_buffer = StorageBuffer::new(&state.rt_device, bytemuck::cast_slice(&light_triangles), Some("LightTriangles"));

        uniforms.num_lights = light_triangles.len() as u32;

        let uniform_buffer: UniformBuffer = UniformBuffer::new(&state.rt_device, bytemuck::cast_slice(&[uniforms]), Some("Uniforms"));

        let rt_bgl = RTStep::get_rt_bgl(&state.rt_device);

        let rt_bind_group = state.rt_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RT Bind Group"),
            layout: &rt_bgl,
            entries: &[
                // wgpu::BindGroupEntry {
                //     binding: 0,
                //     resource: uniform_buffer.buffer().buffer.as_entire_binding(),
                // },
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.output_texture_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.tlas_package.as_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: materials_buffer.buffer().buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: vertices_buffer.buffer().buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: indices_buffer.buffer().buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: instance_info_buffer.buffer().buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: light_triangles_buffer.buffer().buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(
                        &state.depth_texture_rt_view,
                    ),
                },
            ],
        });

        let texture_bgl = RTStep::get_texture_bgl(&state.rt_device, &scene);

        let texture_bind_group = state.rt_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(
                        &scene.textures.iter().map(|texture| &texture.view).collect::<Vec<_>>()
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::SamplerArray(
                        &scene.textures.iter().map(|texture| &texture.sampler).collect::<Vec<_>>()
                    ),
                },
            ],
        });

        self.rt_bind_group = Some(rt_bind_group);
        self.texture_bind_group = Some(texture_bind_group);
    }
}

impl RenderStep for RTStep {
    fn create(wgpu_state: &mut wgpu_util::WGPUState, scene: &scene::Scene) -> Self {
        let rt_bgl = RTStep::get_rt_bgl(&wgpu_state.rt_device);
        let texture_bgl = RTStep::get_texture_bgl(&wgpu_state.rt_device, &scene);
        let uniform_bgl = RTStep::get_uniform_bgl(&wgpu_state.rt_device);

        let mut blases = Vec::new();

        let mut encoder = wgpu_state.rt_device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let vertex_buffer = wgpu_state.rt_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&scene.vertices),
            usage:  wgpu::BufferUsages::BLAS_INPUT | 
                    wgpu::BufferUsages::STORAGE | 
                    wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = wgpu_state.rt_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(&scene.indices),
            usage: wgpu::BufferUsages::BLAS_INPUT,
        });

        let mut mesh_size_descs = Vec::new();

        for (i, mesh) in scene.meshes.iter().enumerate() {
            let mut tri_size_descs = Vec::new();
            for _ in (0..mesh.index_count).step_by(3) {
                let size_desc = wgpu::BlasTriangleGeometrySizeDescriptor {
                    vertex_format: wgpu::VertexFormat::Float32x3,
                    vertex_count: 0 as u32,
                    index_format: Some(wgpu::IndexFormat::Uint32),
                    index_count: Some(3),
                    flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
                };

                tri_size_descs.push(size_desc);
            }
            // let size_desc = wgpu::BlasTriangleGeometrySizeDescriptor {
            //     vertex_format: wgpu::VertexFormat::Float32x3,
            //     vertex_count: (mesh.vertex_count) as u32,
            //     index_format: Some(wgpu::IndexFormat::Uint32),
            //     index_count: Some(mesh.index_count),
            //     flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
            // };

            mesh_size_descs.push(tri_size_descs);
        }

        for (i, mesh) in scene.meshes.iter().enumerate() {
            let blas = wgpu_state.rt_device.create_blas(
                &wgpu::CreateBlasDescriptor {
                    label: Some(format!("BLAS {}", i).as_str()),
                    flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE 
                        | wgpu::AccelerationStructureFlags::ALLOW_RAY_HIT_VERTEX_RETURN,
                    update_mode: wgpu::AccelerationStructureUpdateMode::Build,
                },
                wgpu::BlasGeometrySizeDescriptors::Triangles {
                    descriptors: mesh_size_descs[i].clone(),
                }
            );

            blases.push(blas);
        }

        let mut blas_build_entries = Vec::new();
        for (i, mesh) in scene.meshes.iter().enumerate() {
            let mut tri_geoms = Vec::new();
            for j in (0..mesh.index_count).step_by(3) {
                let tri_geom = wgpu::BlasTriangleGeometry {
                    size: &mesh_size_descs[i][(j / 3) as usize],
                    vertex_buffer: &vertex_buffer,
                    first_vertex: 0 as u32,
                    vertex_stride: std::mem::size_of::<uniforms::Vertex>() as u64,
                    index_buffer: Some(&index_buffer),
                    first_index: Some((mesh.index_offset + j) as u32),
                    transform_buffer: None,
                    transform_buffer_offset: None,
                };

                tri_geoms.push(tri_geom);
            }

            blas_build_entries.push(wgpu::BlasBuildEntry {
                blas: &blases[i],
                geometry: wgpu::BlasGeometries::TriangleGeometries(tri_geoms),
            })    
        }

        encoder.build_acceleration_structures(
            blas_build_entries.iter(),
            std::iter::empty()
        );

        wgpu_state.rt_queue.submit(Some(encoder.finish()));
        
        let shader = wgpu_state.rt_device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor::preprocess_wgsl!("shaders/shader_main.wgsl").into()),
        });

        let render_pipeline_layout = wgpu_state.rt_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&rt_bgl, &texture_bgl, &uniform_bgl],
            push_constant_ranges: &[],
        });

        let render_pipeline = wgpu_state.rt_device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pipeline for shader_main.wgsl"),
            layout: Some(&render_pipeline_layout),
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        let tlas = wgpu_state.rt_device.create_tlas(&wgpu::CreateTlasDescriptor {
            label: None,
            max_instances: 10000 as u32,
            flags: 
                wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE | 
                wgpu::AccelerationStructureFlags::ALLOW_RAY_HIT_VERTEX_RETURN,
                
            update_mode: wgpu::AccelerationStructureUpdateMode::Build,
        });

        let mut tlas_package = wgpu::TlasPackage::new(tlas);
        
        let mut this = Self {
            pipeline: render_pipeline,
            bind_groups: Vec::new(),
            blases: blases,
            tlas_package: tlas_package,
            output_texture_view: wgpu_state.blit_storage_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            rt_bind_group: None,
            texture_bind_group: None,
        };
        this
    }

    fn update(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene) {
        let mut uniforms = uniforms::Uniforms::new(&scene.camera);

        let uniform_buffer: UniformBuffer = UniformBuffer::new(&state.rt_device, bytemuck::cast_slice(&[uniforms]), Some("Uniforms"));

        let uniform_bgl = Self::get_uniform_bgl(&state.rt_device);

        let uniform_bind_group = state.rt_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.buffer().buffer.as_entire_binding(),
                }
            ],
        });

        if self.rt_bind_group.is_none() || self.texture_bind_group.is_none() {
            self.create_static_bind_groups(state, scene);
        }

        self.bind_groups = vec![
            self.rt_bind_group.clone().expect("the bind group doesn't exist"), 
            self.texture_bind_group.clone().expect("the bind group doesn't exist"),
            uniform_bind_group,
        ];
    }

    fn render(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene, encoder: &mut wgpu::CommandEncoder, output: Option<&wgpu::SurfaceTexture>) {
        for (i, blas) in self.blases.iter().enumerate() {
            // Update

            let mat_idx = scene.meshes[i].material_index;

            self.tlas_package[i] = Some(TlasInstance::new(
                blas,
                Mat4::from_translation(Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                })
                .transpose()
                .to_cols_array()[..12]
                    .try_into()
                    .unwrap(),
                mat_idx,
                0xff,
            ));
        }

        encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&self.tlas_package));

        {
            let mut render_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });


            render_pass.set_pipeline(&self.pipeline);

            for (i, bind_group) in self.bind_groups.iter().enumerate() {
                render_pass.set_bind_group(i as u32, Some(bind_group), &[]);
            }

            render_pass.dispatch_workgroups(state.config.width / shader_definitions::WORKGROUP_DIM, state.config.height / shader_definitions::WORKGROUP_DIM, 1);
        }
    }
}
