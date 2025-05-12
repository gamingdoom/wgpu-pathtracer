mod rt;
mod blit;
mod rayproject;

pub use rt::RTStep;
pub use blit::BlitStep;
pub use rayproject::RayprojectStep;

use std::num::NonZeroU32;

use glam::{Vec3, Mat4};
use wgpu::{util::DeviceExt, TlasInstance};

use crate::{scene, shaders::shader_definitions, uniforms, wgpu_buffer::{BufferType, StorageBuffer, UniformBuffer}, wgpu_util};

pub trait RenderStep {
    fn create(state: &mut wgpu_util::WGPUState, scene: &scene::Scene) -> Self;
    fn update(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene);
    fn render(&mut self, state: &mut wgpu_util::WGPUState, scene: &scene::Scene, encoder: &mut wgpu::CommandEncoder, output: Option<&wgpu::SurfaceTexture>);
}
