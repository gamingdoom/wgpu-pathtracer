extern crate image_dds;
extern crate oidn_wgpu_interop;

mod wgpu_util;
mod wgpu_buffer;
mod window;
mod raytracer;
mod uniforms;
mod scene;
mod camera;
mod shaders;
mod texture;
mod render_steps;
mod shader;

fn main() {
    //env_logger::init();
    env_logger::builder()
        //.filter_level(log::LevelFilter::Trace)
        .init();

    shaders::shader_definitions::WORKGROUP_DIM;

    window::run();
}
