//!#include "uniforms.wgsl"
//!#include "random.wgsl"

const PI = 3.14159265359;
const EPSILON = 0.0001;

fn random_point_on_triangle(tri: Triangle) -> vec3<f32> {
    let r1 = rand_float();
    let r2 = rand_float();

    let s1 = sqrt(r1);

    let x = tri.v0.x * (1.0 - s1) + tri.v1.x * s1 * (1.0 - r2) + tri.v2.x * s1 * r2;
    let y = tri.v0.y * (1.0 - s1) + tri.v1.y * s1 * (1.0 - r2) + tri.v2.y * s1 * r2;
    let z = tri.v0.z * (1.0 - s1) + tri.v1.z * s1 * (1.0 - r2) + tri.v2.z * s1 * r2;

    return vec3<f32>(x, y, z);
}

fn equals(a: vec3<f32>, b: vec3<f32>) -> bool {
    return  abs(a.x - b.x) < EPSILON && 
            abs(a.y - b.y) < EPSILON && 
            abs(a.z - b.z) < EPSILON;
}

fn luminance(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn rotate_point(quat: vec4<f32>, point: vec3<f32>) -> vec3<f32> {
    let q_axis = quat.xyz;
    return 2.0 * dot(q_axis, point) * q_axis + (quat.w * quat.w - dot(q_axis, q_axis)) * point + 2.0 * quat.w * cross(q_axis, point);
}

fn sample_texture_rgba(idx: u32, uv: vec2<f32>) -> vec4<f32> {
    let rgba = textureSampleLevel(textures[idx], samplers[idx], uv, 0.0);
    return rgba;
}

fn sample_texture_color(idx: u32, uv: vec2<f32>) -> vec3<f32> {
    let rgba = textureSampleLevel(textures[idx], samplers[idx], uv, 0.0);
    return vec3<f32>(rgba.x, rgba.y, rgba.z);
}

fn sample_texture_float(idx: u32, uv: vec2<f32>) -> f32 {
    let rgba = textureSampleLevel(textures[idx], samplers[idx], uv, 0.0);
    return rgba.x;
}

struct SampledMaterial {
    albedo: vec3<f32>,
    alpha: f32,

    tangent_space_normal: vec3<f32>,

    roughness: f32,
    specular: vec3<f32>,
    metallic: f32,

    emissive: vec3<f32>,
    sheen: f32,

    clearcoat_thickness: f32,
    clearcoat_roughness: f32,
    anisotropy: f32,
    anisotropy_rotation: f32,

    transmission_weight: f32,
    ior: f32,
};
