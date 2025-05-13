use std::mem;

use wgpu::wgc::api::Vulkan;
use winit::{
    event::{self, *},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use crate::{render_steps::RenderStep, wgpu_util};
use crate::raytracer;

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("WGPU-pathtracer")
        .build(&event_loop)
        .unwrap();

    let mut raytracer = raytracer::Raytracer::new(wgpu_util::WGPUState::new(&window));

    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == raytracer.wgpu_state.window().id() => if !raytracer.input(event) {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        //raytracer.wgpu_state.resize(*physical_size);
                        raytracer.resize(*physical_size);
                        //raytracer = raytracer::Raytracer::new(wgpu_util::WGPUState::new(&raytracer.wgpu_state.window()));
                        //create_new_raytracer(raytracer);

                    },
                    WindowEvent::RedrawRequested => {
                        raytracer.wgpu_state.window().request_redraw();

                        raytracer.update();

                        match raytracer.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => control_flow.exit(),//raytracer.wgpu_state.resize(raytracer.wgpu_state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }).unwrap();

}
