use std::iter;

use glam::{Mat4, Vec3};
use wgpu::wgc::api::Vulkan;
use wgpu::CommandEncoder;
use wgpu::PollType;
use wgpu::SurfaceTexture;
use wgpu::TlasInstance;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::window::CursorGrabMode;

use ash::vk;

use crate::camera;
use crate::render_steps;
use crate::render_steps::RenderStep;
use crate::texture;
use crate::wgpu_util;
use crate::scene;
use crate::shaders::shader_definitions;

pub struct Raytracer<'a> {
    pub wgpu_state: wgpu_util::WGPUState<'a>,
    pub window_cursor_grabbed: bool,
    pub scene: scene::Scene,

    pub rt_render_step: render_steps::RTStep,
    pub rayproject_render_step: render_steps::RayprojectStep,
    pub blit_render_step: render_steps::BlitStep,
    pub raytracer_submission_index: Option<wgpu::SubmissionIndex>
}

impl<'a> Raytracer<'a> {
    pub fn new(mut wgpu_state: wgpu_util::WGPUState<'a>) -> Raytracer<'a> {
        let camera = camera::Camera::new(
            wgpu_state.window().inner_size(),
            60.0,
            Vec3::new(0.0, 0.0, 0.0),
        );

        let mut scene = scene::Scene::new(&wgpu_state, camera);

        scene.load_obj(&wgpu_state, "res/classroom/classroom.obj");
        //scene.load_obj(&wgpu_state, "res/minecraft/minecraft.obj");
        //scene.load_obj(&wgpu_state, "res/sports_car/sportsCar.obj");
        //scene.load_obj(&wgpu_state, "res/salle_de_bain/salle_de_bain.obj");
        //scene.load_obj(&wgpu_state, "res/living_room/living_room.obj");
        //scene.load_obj(&wgpu_state, "res/fireplace_room/fireplace_room.obj");
        //scene.load_obj(&wgpu_state, "res/cornell_box_pbr.obj");
        //scene.load_obj(&wgpu_state, "res/san_miguel/san-miguel.obj");
        //scene.load_obj(&wgpu_state, "res/bedroom/iscv2.obj");

        //let (render_pipeline, blases) = scene.create_resources(&wgpu_state.device, &wgpu_state.queue);

        //scene.blases = blases;

        let mut rt_step = render_steps::RTStep::create(&mut wgpu_state, &scene);
        let rayproject_step = render_steps::RayprojectStep::create(&mut wgpu_state, &scene);
        let blit_step = render_steps::BlitStep::create(&mut wgpu_state, &scene);

        //rt_step.set_output_texture(&rayproject_step.latest_real_frame_rt);
        rt_step.output_texture_view = rayproject_step.latest_real_frame_rt.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            wgpu_state: wgpu_state,
            window_cursor_grabbed: false,
            scene,
            rt_render_step: rt_step,
            rayproject_render_step: rayproject_step,
            blit_render_step: blit_step,
            raytracer_submission_index: None,
            //rt_pipeline: render_pipeline,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        //let mut encoder = self.wgpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        //let (rt_bind_group, texture_bind_group) = self.scene.update_resources(&self.wgpu_state);
        //let (rt_bind_group, texture_bind_group) = (self.rt_bind_group, self.texture_bind_group);

        // for (i, blas) in self.scene.blases.iter().enumerate() {
        //     // Update

        //     let mat_idx = self.scene.meshes[i].material_index;

        //     self.wgpu_state.tlas_package[i] = Some(TlasInstance::new(
        //         blas,
        //         Mat4::from_translation(Vec3 {
        //             x: 0.0,
        //             y: 0.0,
        //             z: 0.0,
        //         })
        //         .transpose()
        //         .to_cols_array()[..12]
        //             .try_into()
        //             .unwrap(),
        //         mat_idx,
        //         0xff,
        //     ));
        // }

        // encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&self.wgpu_state.tlas_package));

        // {
        //     let mut render_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        //         label: None,
        //         timestamp_writes: None,
        //     });


        //     render_pass.set_pipeline(&self.rt_pipeline);
        //     render_pass.set_bind_group(0, Some(&rt_bind_group), &[]);
        //     render_pass.set_bind_group(1, Some(&texture_bind_group), &[]);
        //     render_pass.dispatch_workgroups(self.wgpu_state.config.width / shader_definitions::WORKGROUP_DIM, self.wgpu_state.config.height / shader_definitions::WORKGROUP_DIM, 1);
        // }

        // if we arent done with the previous raytracing, return
        if !self.wgpu_state.rt_device.poll(wgpu::PollType::Poll).unwrap().is_queue_empty() {
            return Ok(());
        }

        let mut encoder = self.wgpu_state.rt_device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("RT Encoder") });

        // encoder.copy_texture_to_texture(
        //     &wgpu::TexelCopyTextureInfo {
        //         texture: &self.
        //     }
        // );

        self.scene.prev_camera = self.scene.camera;
        self.rt_render_step.update(&mut self.wgpu_state, &self.scene);
        self.rt_render_step.render(&mut self.wgpu_state, &self.scene, &mut encoder, None);

        // let submit_info = vk::SubmitInfo::default()
        //     .command_buffers(&[encoder.finish()]);

        // unsafe {self.wgpu_state.vk_device.queue_submit(self.wgpu_state.rt_queue, submits, fence);}

        self.raytracer_submission_index = Some(self.wgpu_state.rt_queue.submit(Some(encoder.finish())));
        
        println!("submitted");

        // let mut encoder = self.wgpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Blit, Rayproject Encoder") });

        // self.rayproject_render_step.render(&mut self.wgpu_state, &self.scene, &mut encoder, None);

        // self.blit_render_step.render(&mut self.wgpu_state, &self.scene, &mut encoder, Some(&output));

        // self.wgpu_state.queue.submit(Some(encoder.finish()));
        // output.present();

        Ok(())
    }

    pub fn update(&mut self) {
        self.scene.camera.width = self.wgpu_state.size.width;
        self.scene.camera.height = self.wgpu_state.size.height;
        self.scene.camera.update();

        if self.window_cursor_grabbed {
            self.wgpu_state.resize(self.wgpu_state.size);

            if self.scene.camera.frame % 10 == 0 {
                self.scene.camera.frame = 0;
            }
        }

        self.rayproject_render_step.update(&mut self.wgpu_state, &self.scene);
        
        let output = self.wgpu_state.surface.get_current_texture().unwrap();

        let mut encoder = self.wgpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Blit, Rayproject Encoder") });

        self.rayproject_render_step.render(&mut self.wgpu_state, &self.scene, &mut encoder, None);

        self.blit_render_step.render(&mut self.wgpu_state, &self.scene, &mut encoder, Some(&output));

        self.wgpu_state.pp_queue.submit(Some(encoder.finish()));
        output.present();
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::KeyboardInput {event, ..} => {

                if event.state == winit::event::ElementState::Released {
                    return false;
                }

                let pkey = event.physical_key;
                match pkey {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        let camera_dir = (self.scene.camera.position - self.scene.camera.lookat).normalize();
                        self.scene.camera.position += -camera_dir * self.scene.camera.camera_speed;
                        self.scene.camera.lookat += -camera_dir * self.scene.camera.camera_speed;

                        return true;
                    },
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        let camera_dir = (self.scene.camera.position - self.scene.camera.lookat).normalize();
                        self.scene.camera.position += camera_dir * self.scene.camera.camera_speed;
                        self.scene.camera.lookat += camera_dir * self.scene.camera.camera_speed;

                        return true;
                    },
                    PhysicalKey::Code(KeyCode::KeyQ) => {
                        self.scene.camera.position[1] -= self.scene.camera.camera_speed;
                        self.scene.camera.lookat[1] -= self.scene.camera.camera_speed;

                        return true;
                    },
                    PhysicalKey::Code(KeyCode::KeyE) => {
                        self.scene.camera.position[1] += self.scene.camera.camera_speed;
                        self.scene.camera.lookat[1] += self.scene.camera.camera_speed;

                        return true;
                    },
                    // PhysicalKey::Code(KeyCode::KeyR) => {
                    //     self.wgpu_state.resize(self.wgpu_state.size);
                    //     self.scene.camera.frame = 0;

                    //     return true;
                    // }
                    PhysicalKey::Code(KeyCode::Space) => {
                        self.window_cursor_grabbed = !self.window_cursor_grabbed;
                        let mode = if self.window_cursor_grabbed { CursorGrabMode::Confined } else { CursorGrabMode::None };
                        self.wgpu_state.window.set_cursor_grab(mode).unwrap();
                        self.scene.camera.frame = 0;

                        return true;
                    }
                    _ => {}
                }
            },
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                if self.window_cursor_grabbed {
                    if position.x != self.wgpu_state.config.width as f64 / 2.0 || position.y != self.wgpu_state.config.height as f64 / 2.0 {
                        //if (theta_x + event.motion.xrel * 0.01f < M_PI/2.0f && theta_x + event.motion.xrel * 0.01f > -M_PI/2.0f) {
                        self.scene.camera.theta_x += (position.x - self.wgpu_state.config.width as f64 / 2.0).atan() as f32 * 0.0025;
                        //}
                        if self.scene.camera.theta_y - (position.y - self.wgpu_state.config.height as f64 / 2.0).atan() as f32 * 0.01 < std::f32::consts::PI/2.0 && self.scene.camera.theta_y - (position.y - self.wgpu_state.config.height as f64 / 2.0).atan() as f32 * 0.01 > -std::f32::consts::PI/2.0 {
                            self.scene.camera.theta_y -= (position.y - self.wgpu_state.config.height as f64 / 2.0).atan() as f32 * 0.0025;
                        }

                        self.wgpu_state.window.set_cursor_position(winit::dpi::PhysicalPosition::new(self.wgpu_state.config.width as f64 / 2.0, self.wgpu_state.config.height as f64 / 2.0)).unwrap();

                        return true
                    }
                }
            },
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        if *y < 0.0 && self.scene.camera.camera_speed == 0.0 {
                            return false;
                        }
                        self.scene.camera.camera_speed += y;
                        return true;
                    },
                    _ => {}
                }
            },
            _ => {}
        };

        false
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 && self.wgpu_state.size != size {
            // wait queues
            self.wgpu_state.rt_device.poll(PollType::Wait).unwrap();
            self.wgpu_state.device.poll(PollType::Wait).unwrap();

            self.wgpu_state.size = size;
            self.wgpu_state.config.width = size.width;
            self.wgpu_state.config.height = size.height;
            self.blit_render_step.update(&mut self.wgpu_state, &self.scene);

            let latest_real_frame_desc = &wgpu::TextureDescriptor {
                label: Some("prev_frame"),
                size: wgpu::Extent3d {
                    width: self.wgpu_state.config.width,
                    height: self.wgpu_state.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            };

            self.rayproject_render_step.latest_real_frame = self.wgpu_state.device.create_texture(latest_real_frame_desc);
    
            self.rayproject_render_step.latest_real_frame_rt = unsafe { self.wgpu_state.rt_device.create_texture_from_hal::<Vulkan>(
                self.wgpu_state.rt_device.as_hal::<Vulkan, _, _>(|dev| wgpu::hal::vulkan::Device::texture_from_raw(
                    self.rayproject_render_step.latest_real_frame.as_hal::<Vulkan, _, _>(|tex| {
                        tex.unwrap().raw_handle()
                    }), 
                    &wgpu::hal::TextureDescriptor {
                        label: Some("prev_frame"),
                        size: wgpu::Extent3d {
                            width: self.wgpu_state.config.width,
                            height: self.wgpu_state.config.height,
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

            self.rt_render_step.output_texture_view = self.rayproject_render_step.latest_real_frame_rt.create_view(&wgpu::TextureViewDescriptor::default());

            self.wgpu_state.resize_2();

            // self.wgpu_state.size = size;
            // self.wgpu_state.config.width = size.width;
            // self.wgpu_state.config.height = size.height;
            // self.wgpu_state.resize_2();

            // let mut rt_step = render_steps::RTStep::create(&mut self.wgpu_state, &self.scene);
            // let rayproject_step = render_steps::RayprojectStep::create(&mut self.wgpu_state, &self.scene);
            // let blit_step = render_steps::BlitStep::create(&mut self.wgpu_state, &self.scene);
    
            // rt_step.set_output_texture(&rayproject_step.latest_real_frame_rt);

            // self.rt_render_step = rt_step;
            // self.rayproject_render_step = rayproject_step;
            // self.blit_render_step = blit_step;

            // self.wgpu_state.resize();

            // self.blit_render_step.update(&mut self.wgpu_state, &self.scene);
            // self.rayproject_render_step.resize(&mut self.wgpu_state);
            //self.rt_render_step.output_texture_view = self.rayproject_render_step.latest_real_frame_rt.create_view(&wgpu::TextureViewDescriptor::default());

        }
    }
}