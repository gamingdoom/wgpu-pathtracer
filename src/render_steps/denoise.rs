use std::sync::Arc;

use oidn::sys::{OIDNFormat_OIDN_FORMAT_FLOAT3, OIDNQuality_OIDN_QUALITY_BALANCED, OIDNQuality_OIDN_QUALITY_HIGH};
use oidn_wgpu_interop::SharedBuffer;
use wgpu::{Extent3d, PollType, TexelCopyBufferInfo};

use crate::shaders::shader_definitions::USE_DENOISER;
use crate::{scene, wgpu_util};

use crate::render_steps::RenderStep;

pub struct DenoiseStep {
    pub input_texture: wgpu::Texture,
    pub input_tv: wgpu::TextureView,
    pub output_texture: Option<wgpu::Texture>,
    
    buffer: SharedBuffer,

    pub normal_buffer: Option<Arc<SharedBuffer>>,
    pub albedo_buffer: Option<Arc<SharedBuffer>>,
}

impl RenderStep for DenoiseStep {
    fn create(state: &mut wgpu_util::WGPUState, scene: &scene::Scene) -> Self {
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
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let mut input_texture: wgpu::Texture;
        if crate::shaders::shader_definitions::USE_DENOISER {
            input_texture = state.rt_device.create_texture(latest_real_frame_desc);
        } else {
            input_texture = state.latest_real_frame_rt.as_ref().unwrap().clone();
        }

        // Round up to nearest 256
        let tex_bytes_per_row = (256.0 * ((state.config.width * 4 * 4) as f32 / 256.0).ceil()) as u32;
        let tex_num_rows = state.config.height;
        let tex_num_bytes = tex_bytes_per_row * tex_num_rows;

        let buffer = state.oidn_device.allocate_shared_buffers((tex_num_bytes) as u64).unwrap();

        Self {
            input_tv: input_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            input_texture,
            output_texture: None,

            buffer,

            normal_buffer: None,
            albedo_buffer: None,
        }
    }

    fn update(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene) {

    }

    fn render(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene, encoder: &mut wgpu::CommandEncoder, output: Option<&wgpu::SurfaceTexture>) {
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            // Intel OIDN

            // Create a new buffer and copy the output texture to it.
            let out_tex = state.latest_real_frame_rt.as_ref().unwrap();

            // Round up to nearest 256
            let tex_bytes_per_row = (256.0 * ((state.config.width * 4 * 4) as f32 / 256.0).ceil()) as u32;
            let tex_num_rows = state.config.height;
            let tex_num_bytes = tex_bytes_per_row * tex_num_rows;

            // let buffer = state.oidn_device.allocate_shared_buffers((tex_num_bytes) as u64).unwrap();

            let mut copy_encoder = state.rt_device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Copy Encoder") });

            copy_encoder.copy_texture_to_buffer(
                self.input_texture.as_image_copy(),
                TexelCopyBufferInfo {
                    buffer: &self.buffer.wgpu_buffer(),
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(tex_bytes_per_row),
                        rows_per_image: Some(tex_num_rows),
                    }
                },
                Extent3d {
                    width: state.config.width as u32,
                    height: state.config.height as u32,
                    depth_or_array_layers: 1,
                }
            );

            let submission_idx = state.rt_queue.submit(Some(copy_encoder.finish()));

            state.rt_device.poll(PollType::Wait).unwrap();

            // oidn::RayTracing::new(&state.oidn_device.oidn_device())
            //     .srgb(true)
            //     .image_dimensions(state.config.width as usize, state.config.height as usize)
            //     .filter_in_place_buffer(&mut buffer.oidn_buffer_mut())
            //     .expect("filter config error");

            let filter = unsafe { oidn::sys::oidnNewFilter(state.oidn_device.oidn_device().raw(), b"RT\0" as *const _ as _) };
            unsafe { 
                oidn::sys::oidnSetFilterImage(
                    filter,
                    c"color" as *const _ as _,
                    self.buffer.oidn_buffer().raw(),
                    OIDNFormat_OIDN_FORMAT_FLOAT3,
                    state.config.width as usize,
                    state.config.height as usize,
                    0,
                    tex_bytes_per_row as usize / state.config.width as usize,
                    0
                );

                if let Some(normal_buffer) = &self.normal_buffer {
                    oidn::sys::oidnSetFilterImage(
                        filter,
                        c"normal" as *const _ as _,
                        normal_buffer.oidn_buffer().raw(),
                        OIDNFormat_OIDN_FORMAT_FLOAT3,
                        state.config.width as usize,
                        state.config.height as usize,
                        0,
                        16,
                        0
                    );
                }

                if let Some(albedo_buffer) = &self.albedo_buffer {
                    oidn::sys::oidnSetFilterImage(
                        filter,
                        c"albedo" as *const _ as _,
                        albedo_buffer.oidn_buffer().raw(),
                        OIDNFormat_OIDN_FORMAT_FLOAT3,
                        state.config.width as usize,
                        state.config.height as usize,
                        0,
                        16,
                        0
                    );
                }

                oidn::sys::oidnSetFilterImage(
                    filter,
                    c"output" as *const _ as _,
                    self.buffer.oidn_buffer().raw(),
                    OIDNFormat_OIDN_FORMAT_FLOAT3,
                    state.config.width as usize,
                    state.config.height as usize,
                    0,
                    16,
                    0
                );
                oidn::sys::oidnSetFilterInt(filter, c"quality" as *const _ as _, OIDNQuality_OIDN_QUALITY_HIGH.try_into().unwrap());
                oidn::sys::oidnSetFilterBool(filter, c"hdr" as *const _ as _, true);
                
                // Makes image worse for some reason?
                //oidn::sys::oidnSetFilterBool(filter, c"cleanAux" as *const _ as _, true);
                
                oidn::sys::oidnSetFilterBool(filter, c"srgb" as *const _ as _, false);

                oidn::sys::oidnCommitFilter(filter);

                // TODO this is undefined behaviour.
                oidn::sys::oidnExecuteFilterAsync(filter);

                oidn::sys::oidnReleaseFilter(filter);
            };

            if let Err(e) = state.oidn_device.oidn_device().get_error() {
                println!("oidn error: {}", e.1);
                //panic!()
            }

            let mut copy_encoder = state.rt_device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Copy Encoder") });

            copy_encoder.copy_buffer_to_texture(
                TexelCopyBufferInfo {
                    buffer: &self.buffer.wgpu_buffer(),
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(tex_bytes_per_row),
                        rows_per_image: Some(tex_num_rows),
                    }
                },
                out_tex.as_image_copy(),
                Extent3d {
                    width: state.config.width as u32,
                    height: state.config.height as u32,
                    depth_or_array_layers: 1,
                }
            );

            state.rt_queue.submit(Some(copy_encoder.finish()));

            state.rt_device.poll(PollType::Wait).unwrap();
            
            //std::mem::forget(buffer);
        }
    }

    fn resize(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene) {
        // Round up to nearest 256
        let tex_bytes_per_row = (256.0 * ((state.config.width * 4 * 4) as f32 / 256.0).ceil()) as u32;
        let tex_num_rows = state.config.height;
        let tex_num_bytes = tex_bytes_per_row * tex_num_rows;

        let buffer = state.oidn_device.allocate_shared_buffers((tex_num_bytes) as u64).unwrap();

        self.buffer = buffer.into();
    }
}