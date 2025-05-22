//!#pragma once
//!#include "uniforms.wgsl"

//!#define pub
//!#include "shader_definitions.rs"

@group(0) @binding(0)
var output: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(1)
var acc_struct: acceleration_structure;

@group(0) @binding(2)
var<storage, read> materials: array<Material>;

@group(0) @binding(3)
var<storage, read> vertices: array<Vertex>;

@group(0) @binding(4)
var<storage, read> indices: array<u32>;
// var<storage, read> vertex_infos: array<u32>;

@group(0) @binding(5)
var<storage, read> instance_infos: array<InstanceInfo>;

@group(0) @binding(6)
var<storage, read> lights: array<Triangle>;

@group(0) @binding(7)
var depth_output: texture_storage_2d<r32float, write>;


@group(1) @binding(0)
var textures: binding_array<texture_2d<f32>, MAX_TEXTURES>;

@group(1) @binding(1)
var samplers: binding_array<sampler, MAX_TEXTURES>;


@group(2) @binding(0)
var<uniform> uniforms: Uniforms;