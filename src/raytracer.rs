use std::iter;

use glam::{Mat4, Vec3};
use wgpu::TlasInstance;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::window::CursorGrabMode;

use crate::camera;
use crate::texture;
use crate::wgpu_util;
use crate::scene;
use crate::shaders::shader_definitions;

pub struct Raytracer<'a> {
   pub wgpu_state: wgpu_util::WGPUState<'a>,
   pub window_cursor_grabbed: bool,
   pub scene: scene::Scene,

   pub rt_pipeline: wgpu::ComputePipeline,
}

impl<'a> Raytracer<'a> {
    pub fn new(wgpu_state: wgpu_util::WGPUState<'a>) -> Raytracer<'a> {
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

        let (render_pipeline, blases) = scene.create_resources(&wgpu_state.device, &wgpu_state.queue);

        scene.blases = blases;


        Self {
            wgpu_state,
            window_cursor_grabbed: false,
            scene,
            rt_pipeline: render_pipeline,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.wgpu_state.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.wgpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
        
        let (rt_bind_group, texture_bind_group) = self.scene.update_resources(&self.wgpu_state);
        //let (rt_bind_group, texture_bind_group) = (self.rt_bind_group, self.texture_bind_group);

        for (i, blas) in self.scene.blases.iter().enumerate() {
            // Update

            let mat_idx = self.scene.meshes[i].material_index;            

            self.wgpu_state.tlas_package[i] = Some(TlasInstance::new(
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

        encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&self.wgpu_state.tlas_package));

        {
            let mut render_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });


            render_pass.set_pipeline(&self.rt_pipeline);
            render_pass.set_bind_group(0, Some(&rt_bind_group), &[]);
            render_pass.set_bind_group(1, Some(&texture_bind_group), &[]);
            render_pass.dispatch_workgroups(self.wgpu_state.config.width / shader_definitions::WORKGROUP_DIM, self.wgpu_state.config.height / shader_definitions::WORKGROUP_DIM, 1);
        }

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
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            blit_pass.set_pipeline(&self.wgpu_state.blit_pipeline);
            blit_pass.set_bind_group(0, Some(&self.wgpu_state.blit_bind_group), &[]);
            blit_pass.draw(0..3, 0..1);
        }
    
        self.wgpu_state.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn update(&mut self) {
        self.scene.camera.width = self.wgpu_state.size.width;
        self.scene.camera.height = self.wgpu_state.size.height;
        self.scene.camera.update();
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
                    PhysicalKey::Code(KeyCode::Space) => {
                        self.window_cursor_grabbed = !self.window_cursor_grabbed;
                        let mode = if self.window_cursor_grabbed { CursorGrabMode::Confined } else { CursorGrabMode::None };
                        self.wgpu_state.window.set_cursor_grab(mode).unwrap();

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
}