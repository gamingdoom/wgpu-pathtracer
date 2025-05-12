use std::num::NonZeroU32;

use glam::{Vec3, Mat4};
use wgpu::{util::DeviceExt, wgc::api::Vulkan, Device, ImageSubresourceRange, PollType, TlasInstance};

use crate::{scene, shaders::shader_definitions, uniforms, wgpu_buffer::{BufferType, StorageBuffer, UniformBuffer}, wgpu_util};
use crate::render_steps::RenderStep;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RayprojectUniforms {
    pub camera: uniforms::Camera,
    pub prev_camera: uniforms::Camera
}

pub struct RayprojectStep {
    pipeline: wgpu::ComputePipeline,
    bind_groups: Vec<wgpu::BindGroup>,

    pub latest_real_frame: wgpu::Texture,
    pub latest_real_frame_rt: wgpu::Texture,
}

impl RayprojectStep {
    fn get_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl for rayproject.wgsl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }
}

impl RenderStep for RayprojectStep {
    fn create(state: &mut wgpu_util::WGPUState, scene: &scene::Scene) -> Self {
        let bgl = Self::get_bgl(&state.device);

        let shader = state.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor::preprocess_wgsl!("shaders/rayproject.wgsl").into()),
        });

        let pipeline_layout = state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout for rayproject.wgsl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = state.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pipeline for rayproject.wgsl"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        let latest_real_frame_desc = &wgpu::TextureDescriptor {
            label: Some("prev_frame"),
            size: wgpu::Extent3d {
                width: state.config.width,
                height: state.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let latest_real_frame = state.device.create_texture(latest_real_frame_desc);

        let raw_tex = unsafe { latest_real_frame.as_hal::<Vulkan, _, _>(|tex| {
            tex.unwrap().raw_handle()
        }) };

        let latest_real_frame_rt = unsafe { state.rt_device.create_texture_from_hal::<Vulkan>(
            state.rt_device.as_hal::<Vulkan, _, _>(|dev| wgpu::hal::vulkan::Device::texture_from_raw(
                raw_tex, 
                &wgpu::hal::TextureDescriptor {
                    label: Some("prev_frame"),
                    size: wgpu::Extent3d {
                        width: state.config.width,
                        height: state.config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_formats: (&[]).to_vec(),
                    usage: wgpu::TextureUses::STORAGE_READ_ONLY | wgpu::TextureUses::COPY_DST,
                    memory_flags: wgpu::hal::MemoryFlags::empty()
                }, 
                Some(Box::new(|| {}))
            )),
            latest_real_frame_desc
        ) };

        // let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Init Encoder") });
        // encoder.clear_texture(&latest_real_frame, &ImageSubresourceRange {
        //     ..Default::default()
        // });
        // state.pp_queue.submit(Some(encoder.finish()));
        // state.device.poll(PollType::Wait).unwrap();

        Self {
            pipeline,
            bind_groups: Vec::new(),
            latest_real_frame,
            latest_real_frame_rt,
        }
    }

    fn update(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene) {
        let bgl = Self::get_bgl(&state.device);

        let uniforms = RayprojectUniforms {
            camera: uniforms::Camera::new(&scene.camera),
            prev_camera: uniforms::Camera::new(&scene.prev_camera),
        };

        let uniform_buffer = UniformBuffer::new(&state.device, bytemuck::cast_slice(&[uniforms]), Some("Uniforms"));

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group for blit.wgsl"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &state.blit_storage_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.latest_real_frame.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &state.depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: uniform_buffer.buffer().buffer.as_entire_binding(),
                },
            ],
        });

        self.bind_groups = vec![bind_group];
    }

    fn render(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene, encoder_main: &mut wgpu::CommandEncoder, output: Option<&wgpu::SurfaceTexture>) {
        let mut is_prev_done = state.device.poll(PollType::Poll).unwrap().is_queue_empty();

        while !is_prev_done {
            is_prev_done = state.device.poll(PollType::Poll).unwrap().is_queue_empty();

            if is_prev_done {
                break;
            }

            let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Rayproject Main Encoder"),
            });
    
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
                
                println!("reprojected");
            }
    
            let idx = state.pp_queue.submit(Some(encoder.finish()));
            state.device.poll(PollType::WaitForSubmissionIndex(idx)).unwrap();
        }

        // copy the result to the prev_frame
        // encoder_main.copy_texture_to_texture(
        //     wgpu::TexelCopyTextureInfo {
        //         texture: &state.blit_storage_texture,
        //         origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        //         aspect: wgpu::TextureAspect::All,
        //         mip_level: 0,
        //     },
        //     wgpu::TexelCopyTextureInfo {
        //         texture: &self.prev_frame,
        //         origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        //         aspect: wgpu::TextureAspect::All,
        //         mip_level: 0,
        //     },
        //     wgpu::Extent3d {
        //         width: state.config.width,
        //         height: state.config.height,
        //         depth_or_array_layers: 1,
        //     },
        // );
    }
}