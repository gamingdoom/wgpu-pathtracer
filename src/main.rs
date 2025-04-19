
mod wgpu_util;
mod wgpu_buffer;
mod window;
mod raytracer;
mod uniforms;
mod scene;
mod camera;
mod shaders;
mod texture;

fn main() {
    env_logger::init();

    shaders::shader_definitions::WORKGROUP_DIM;

    pollster::block_on(window::run());
}
