use std::mem;

use wgpu::wgc::api::Vulkan;
// use winit::{
//     event::{self, *},
//     event_loop::EventLoop,
//     keyboard::{KeyCode, PhysicalKey},
//     window::WindowBuilder,
// };

use sdl3::{event::{Event, WindowEvent}, keyboard::Keycode};


use crate::{render_steps::RenderStep, wgpu_util};
use crate::raytracer;

pub fn run() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("WGPU-pathtracer", 800, 600)
        .position_centered()
        .resizable()
        .vulkan()
        .build()
        .unwrap();

    let mut raytracer = raytracer::Raytracer::new(wgpu_util::WGPUState::new(&sdl_context, &window));

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {  
                    win_event: 
                        WindowEvent::PixelSizeChanged(width, height)
                        | WindowEvent::Resized(width, height),
                    .. 
                } => {
                    raytracer.resize((width as u32, height as u32));
                },
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running;
                },
                Event::KeyDown { .. } 
                | Event::MouseWheel { .. } 
                | Event::MouseMotion { .. } => {
                    raytracer.input(&event);
                }
                _ => {}
            }
        }

        raytracer.update();

        match raytracer.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => break 'running,
            Err(wgpu::SurfaceError::OutOfMemory) => break 'running,
            Err(e) => eprintln!("{:?}", e),
        }
    }

    // let event_loop = EventLoop::new().unwrap();
    // let window = WindowBuilder::new()
    //     .with_title("WGPU-pathtracer")
    //     .build(&event_loop)
    //     .unwrap();

    // let mut raytracer = raytracer::Raytracer::new(wgpu_util::WGPUState::new(&window));

    // event_loop.run(move |event, control_flow| {
    //     match event {
    //         Event::WindowEvent {
    //             ref event,
    //             window_id,
    //         } if window_id == raytracer.wgpu_state.window().id() => if !raytracer.input(event) {
    //             match event {
    //                 WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
    //                     event:
    //                         KeyEvent {
    //                             state: ElementState::Pressed,
    //                             physical_key: PhysicalKey::Code(KeyCode::Escape),
    //                             ..
    //                         },
    //                     ..
    //                 } => control_flow.exit(),
    //                 WindowEvent::Resized(physical_size) => {
    //                     //raytracer.wgpu_state.resize(*physical_size);
    //                     raytracer.resize(*physical_size);
    //                     //raytracer = raytracer::Raytracer::new(wgpu_util::WGPUState::new(&raytracer.wgpu_state.window()));
    //                     //create_new_raytracer(raytracer);

    //                 },
    //                 WindowEvent::RedrawRequested => {
    //                     raytracer.wgpu_state.window().request_redraw();

    //                     raytracer.update();

    //                     match raytracer.render() {
    //                         Ok(_) => {}
    //                         Err(wgpu::SurfaceError::Lost) => control_flow.exit(),//raytracer.wgpu_state.resize(raytracer.wgpu_state.size),
    //                         Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
    //                         Err(e) => eprintln!("{:?}", e),
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         _ => {}
    //     }
    // }).unwrap();

}

pub mod create_surface {
    use sdl3::video::Window;
    use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

    // contains the unsafe impl as much as possible by putting it in this module
    struct SyncWindow<'a>(&'a Window);

    unsafe impl<'a> Send for SyncWindow<'a> {}
    unsafe impl<'a> Sync for SyncWindow<'a> {}

    impl<'a> HasWindowHandle for SyncWindow<'a> {
        fn window_handle(&self) -> Result<wgpu::rwh::WindowHandle<'_>, wgpu::rwh::HandleError> {
            self.0.window_handle()
        }
    }
    impl<'a> HasDisplayHandle for SyncWindow<'a> {
        fn display_handle(&self) -> Result<wgpu::rwh::DisplayHandle<'_>, wgpu::rwh::HandleError> {
            self.0.display_handle()
        }
    }

    pub fn create_surface<'a>(
        instance: &wgpu::Instance,
        window: &'a Window,
    ) -> Result<wgpu::Surface<'a>, String> {
        instance
            .create_surface(SyncWindow(&window))
            .map_err(|err| err.to_string())
    }
}

// pub fn run() {
//     let event_loop = EventLoop::new().unwrap();
//     let window = WindowBuilder::new()
//         .with_title("WGPU-pathtracer")
//         .build(&event_loop)
//         .unwrap();

//     let mut raytracer = raytracer::Raytracer::new(wgpu_util::WGPUState::new(&window));

//     event_loop.run(move |event, control_flow| {
//         match event {
//             Event::WindowEvent {
//                 ref event,
//                 window_id,
//             } if window_id == raytracer.wgpu_state.window().id() => if !raytracer.input(event) {
//                 match event {
//                     WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
//                         event:
//                             KeyEvent {
//                                 state: ElementState::Pressed,
//                                 physical_key: PhysicalKey::Code(KeyCode::Escape),
//                                 ..
//                             },
//                         ..
//                     } => control_flow.exit(),
//                     WindowEvent::Resized(physical_size) => {
//                         //raytracer.wgpu_state.resize(*physical_size);
//                         raytracer.resize(*physical_size);
//                         //raytracer = raytracer::Raytracer::new(wgpu_util::WGPUState::new(&raytracer.wgpu_state.window()));
//                         //create_new_raytracer(raytracer);

//                     },
//                     WindowEvent::RedrawRequested => {
//                         raytracer.wgpu_state.window().request_redraw();

//                         raytracer.update();

//                         match raytracer.render() {
//                             Ok(_) => {}
//                             Err(wgpu::SurfaceError::Lost) => control_flow.exit(),//raytracer.wgpu_state.resize(raytracer.wgpu_state.size),
//                             Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
//                             Err(e) => eprintln!("{:?}", e),
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//             _ => {}
//         }
//     }).unwrap();

// }
