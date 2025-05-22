use std::num::NonZeroU32;

use glam::{Vec3, Mat4};
use wgpu::{util::DeviceExt, TlasInstance};

use crate::{scene, shaders::shader_definitions, uniforms, wgpu_buffer::{BufferType, StorageBuffer, UniformBuffer}, wgpu_util};
use crate::render_steps::RenderStep;

pub struct BlitStep {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl BlitStep {
    fn get_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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

        blit_bgl
    }
}

impl RenderStep for BlitStep {
    fn create(state: &mut wgpu_util::WGPUState, scene: &scene::Scene) -> Self {
        let blit_bgl = BlitStep::get_bgl(&state.device);

        let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
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

        let blit_shader = state.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor::preprocess_wgsl!("shaders/blit.wgsl").into()),
        });

        let blit_pipeline_layout = state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout for blit.wgsl"),
            bind_group_layouts: &[&blit_bgl],
            push_constant_ranges: &[],
        });

        let blit_pipeline = state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: state.config.format,
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            multiview: None,
            cache: None,
        });

        let blit_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group for blit.wgsl"),
            layout: &blit_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &state.blit_storage_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        BlitStep {
            pipeline: blit_pipeline,
            bind_group: blit_bind_group,
        }
    }

    fn update(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene) {
        let blit_bgl = BlitStep::get_bgl(&state.device);

        let storage_tex: wgpu::Texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: state.config.width,
                height: state.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
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

        let blit_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        state.blit_storage_texture = storage_tex;

        state.surface.configure(&state.device, &state.config);

        self.bind_group = blit_bind_group;
    }

    fn render(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene, encoder: &mut wgpu::CommandEncoder, output: Option<&wgpu::SurfaceTexture>) {
        let view = output.unwrap().texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut blit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            blit_pass.set_pipeline(&self.pipeline);
            blit_pass.set_bind_group(0, Some(&self.bind_group), &[]);
            blit_pass.draw(0..3, 0..1);
        }
    }
}