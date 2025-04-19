//!#pragma once

struct Camera {
    position: vec3<f32>,
    lookat: vec3<f32>,

    frame: u32,

    pixel_space_x: vec3<f32>,
    pixel_space_y: vec3<f32>,
    first_pixel_pos: vec3<f32>,

    width: u32,
    height: u32,

    fov: f32,
  
    samples_per_pixel: u32,
    max_bounces: u32,
}

struct Uniforms {
    camera: Camera,
    num_lights: u32,
};

// struct Material {
//     albedo: vec3<f32>,
//     roughness: f32,

//     specular: vec3<f32>,
//     metallic: f32,

//     emissive: vec3<f32>,
//     sheen: f32,

//     clearcoat_thickness: f32,
//     clearcoat_roughness: f32,
//     anisotropy: f32,
//     anisotropy_rotation: f32

// };

struct Material {
    albedo_texture_idx: u32,
    roughness_texture_idx: u32,
    metallic_texture_idx: u32,
    emissive_texture_idx: u32,

    specular_texture_idx: u32,
    sheen_texture_idx: u32,
    clearcoat_thickness: f32,
    clearcoat_roughness: f32,

    anisotropy: f32,
    anisotropy_rotation: f32,
    normal_texture_idx: u32,
};

struct Vertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
};

struct InstanceInfo {
    index_offset: u32,
};

struct Triangle {
    v0: vec3<f32>,
    v1: vec3<f32>,
    v2: vec3<f32>,

    normal: vec3<f32>,
};