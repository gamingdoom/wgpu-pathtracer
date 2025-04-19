//!#pragma once
//!#include "uniforms.wgsl"

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var output: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(2)
var acc_struct: acceleration_structure<vertex_return>;

@group(0) @binding(3)
var<storage, read> materials: array<Material>;

@group(0) @binding(4)
var<storage, read> vertices: array<Vertex>;

@group(0) @binding(5)
var<storage, read> indices: array<u32>;
// var<storage, read> vertex_infos: array<u32>;

@group(0) @binding(6)
var<storage, read> instance_infos: array<InstanceInfo>;

@group(0) @binding(7)
var<storage, read> lights: array<Triangle>;

@group(1) @binding(0)
var textures: binding_array<texture_2d<f32>>;

@group(1) @binding(1)
var samplers: binding_array<sampler>;