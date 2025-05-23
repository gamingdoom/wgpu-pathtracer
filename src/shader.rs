use wgpu::{naga::{self, back::{glsl, hlsl}}, util::make_spirv};

use crate::wgpu_util;

pub fn create_shader_compute(state: &wgpu_util::WGPUState, device: &wgpu::Device, source_path: &str) -> wgpu::ShaderModule {
    let source_str = wgsl_preprocessor::preprocess_wgsl!("shaders/shader_main.wgsl");

    let module = naga::front::wgsl::parse_str(source_str.as_str()).unwrap();

    let module_info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .subgroup_stages(naga::valid::ShaderStages::all())
        .subgroup_operations(naga::valid::SubgroupOperationSet::all())
        .validate(&module).unwrap();

    let mut new_source = String::new();

    naga::back::glsl::Writer::new(
            &mut new_source,
            &module,
            &module_info,
            &naga::back::glsl::Options {
                version: glsl::Version::Desktop(460),
                ..Default::default()
            },
            &glsl::PipelineOptions {
                entry_point: "main".to_string(),
                shader_stage: naga::ShaderStage::Compute,
                multiview: None,
            },
            naga::proc::BoundsCheckPolicies::default(),
        ).unwrap()
        .write().unwrap();

    let spirv = jit_spirv::jit_spirv!(
        &new_source,
        glsl,
        comp,
        vulkan1_2,
        entry="main"
    ).unwrap().spv;

    return unsafe { device.create_shader_module_passthrough(wgpu::ShaderModuleDescriptorPassthrough::SpirV (
        wgpu::ShaderModuleDescriptorSpirV {
            label: Some(source_path),
            source: bytemuck::cast_slice(&spirv).into(),
        }
    )) };
}